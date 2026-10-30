import json
import math
import unittest
from unittest.mock import patch
from itertools import permutations
from protocol import lint, generate, precheck, convergence, discrete_reference, check_result, ROOT


def spec(name='scordelis-lo'):
    return json.loads((ROOT/'spec'/f'{name}.json').read_text())


class ProtocolTests(unittest.TestCase):
    def test_all_specs_and_meshes(self):
        for p in (ROOT/'spec').glob('*.json'):
            if p.name == 'schema.json': continue
            s = lint(json.loads(p.read_text()))
            for n in s['mesh_sequence']:
                precheck(generate(s,n))

    def test_missing_references(self):
        s=spec();s['references'].pop()
        with self.assertRaisesRegex(ValueError,'two references'):lint(s)

    def test_empty_justification(self):
        s=spec();s['tolerance']['justification']=' '
        with self.assertRaisesRegex(ValueError,'justification'):lint(s)

    def test_provisional_gate(self):
        s=spec();s['references'][1]['provisional']=True
        with self.assertRaisesRegex(ValueError,'provisional'):lint(s)

    def test_original_mattiasson_table(self):
        s=spec('mattiasson-5');s['references'][1]['value']=.6647
        with self.assertRaisesRegex(ValueError,'two-reference agreement'):lint(s)

    def test_frame_is_not_a_truss(self):
        s=spec('shallow-toggle-truss');s['geometry']['element']='frame'
        with self.assertRaisesRegex(ValueError,'element/reference'):lint(s)

    def test_missing_supported_load(self):
        e=generate(spec(),4);e['model']['loads'].pop()
        with self.assertRaisesRegex(ValueError,'analytic total'):precheck(e)

    def test_unrestrained_model(self):
        e=generate(spec(),4);e['model']['supports']={}
        with self.assertRaisesRegex(ValueError,'rigid-body modes'):precheck(e)

    def test_crown_pin(self):
        with self.assertRaisesRegex(ValueError,'support/reaction contract'):
            precheck(generate(spec(),4,mutant='crown_pin'))

    def test_corners_union(self):
        s=spec('pinched-cylinder');e=generate(s,4)
        c=e['model']['supports']['1']
        self.assertTrue(all(c[k] for k in ['rx','rz','rrx','rry','rrz']))
        with self.assertRaisesRegex(ValueError,'support/reaction contract'):
            precheck(generate(s,4,mutant='missing_corner_symmetry'))

    def test_richardson_and_suspect_classification(self):
        s=spec();v=s['references'][0]['value']
        self.assertAlmostEqual(convergence(s,[v+.004,v+.001,v+.00025])['extrapolated'],v)
        for values in ([v+.01]*3,[v+.001,v+.002,v+.004],[v-.01,v+.005,v-.0025],[float('nan')]*3):
            with self.assertRaises(ValueError):convergence(s,values)
        with self.assertRaisesRegex(ValueError,'suspect'):convergence(s,[-v+.004,-v+.001,-v+.00025])

    def test_informational_references_do_not_change_convergence(self):
        s=spec();v=s['references'][0]['value']
        passing=[v+.004,v+.001,v+.00025]
        failing=[v+.001,v+.002,v+.004]
        baseline=convergence(s,passing)
        admitted=s['references'][:]
        informational=[dict(admitted[0], value=100., provisional=provisional, pass_fail=False)
                       for provisional in (True, False)]
        for refs in permutations(admitted+informational):
            s['references']=list(refs)
            lint(s)
            self.assertEqual(convergence(s,passing),baseline)
            with self.assertRaises(ValueError):convergence(s,failing)

    def test_admitted_reference_order_does_not_change_convergence(self):
        s=spec()
        passing=[-.3064,-.3034,-.30265]
        baseline=convergence(s,passing)
        self.assertEqual(baseline['errors'],
                         [max(abs(v-r['value']) for r in s['references']) for v in passing])
        for refs in permutations(s['references']):
            s['references']=list(refs)
            lint(s)
            with self.subTest(references=[r['value'] for r in refs]):
                self.assertEqual(convergence(s,passing),baseline)
                # Finest mesh passes Altair's gate but misses COMSOL's gate.
                with self.assertRaisesRegex(ValueError,'finest mesh misses reference'):
                    convergence(s,[-.315,-.3105,-.3082])
                # Error decreases toward COMSOL but grows away from Altair.
                with self.assertRaisesRegex(ValueError,'plateau or growing error'):
                    convergence(s,[-.3023,-.3021,-.30205])

    def test_four_mesh_early_reversal(self):
        s=spec();s['mesh_sequence']=[2,4,8,16]
        lint(s)
        with self.assertRaisesRegex(ValueError,'non-asymptotic'):
            convergence(s,[-.3104,-.2984,-.3014,-.30215])
        v=s['references'][0]['value']
        self.assertAlmostEqual(convergence(s,[v+.016,v+.004,v+.001,v+.00025])['extrapolated'],v)
        with self.assertRaisesRegex(ValueError,'non-asymptotic'):
            convergence(s,[v+.016,v+.015,v+.004,v+.001])

    def test_normal_generator_extra_restraint(self):
        original=generate
        def broken(*args, **kwargs):
            e=original(*args, **kwargs)
            e['model']['supports']['1']['rz']=True
            return e
        with patch('protocol.generate', side_effect=broken) as generator:
            with self.assertRaisesRegex(ValueError,'support/reaction contract'):
                precheck(generator(spec(),4))
            self.assertEqual(generator.call_count,1)

    def test_full_support_contract(self):
        for name in ('scordelis-lo','pinched-cylinder'):
            precheck(generate(spec(name),4,full=True))

    def test_discrete_reference_permutations_and_informational_references(self):
        for element in ('truss', 'frame'):
            s=spec('shallow-toggle-'+element)
            value=s['references'][0]['value']
            admitted=s['references'][:]
            informational=[dict(admitted[0], value=100., provisional=provisional, pass_fail=False)
                           for provisional in (True, False)]
            for refs in permutations(admitted+informational):
                s['references']=list(refs)
                lint(s)
                discrete_reference(s,value)
                with self.assertRaisesRegex(ValueError,'discrete response misses reference'):
                    discrete_reference(s,value*(1+2*s['tolerance']['relative']))

    def test_discrete_reference_disagreement_inside_uncertainty(self):
        s=spec('shallow-toggle-frame')
        value=s['references'][0]['value']
        s['references'][1]['value']+=8e-11
        for refs in permutations(s['references']):
            s['references']=list(refs)
            lint(s)
            for response in (value, value+8e-11):
                with self.assertRaisesRegex(ValueError,'discrete response misses reference'):
                    discrete_reference(s,response)

    def test_discrete_reference_uses_declared_tolerance(self):
        s=spec('shallow-toggle-frame')
        value=s['references'][0]['value']*(1+5e-9)
        discrete_reference(s,value)
        s['tolerance']['relative']=1e-9
        lint(s)
        with self.assertRaisesRegex(ValueError,'discrete response misses reference'):
            discrete_reference(s,value)

    def test_discrete_reference_never_admits_continuum(self):
        s=spec('shallow-toggle-frame');v=s['references'][0]['value']
        with self.assertRaisesRegex(ValueError,'discrete reference'):
            convergence(s,[v+1e-8,v+2.5e-9,v+6.25e-10])

    def test_reaction_failure(self):
        e=generate(spec(),4)
        with self.assertRaisesRegex(ValueError,'reactions'):
            check_result(e,dict(reactions=[],displacements=[]))

    def test_balanced_force_with_wrong_moment(self):
        e=generate(spec('mattiasson-1'),1)
        result=dict(reactions=[dict(nodeId=1,rx=0.,rz=5000.,my=0.)],
                    displacements=[dict(nodeId=k,ux=0.,uz=0.,ry=0.) for k in (1,2)])
        with self.assertRaisesRegex(ValueError,'reaction moments'):
            check_result(e,result)

    def test_schema_rejects_typos_and_invalid_geometry(self):
        s=spec();s['material']['youngs']=s['material'].pop('e')
        with self.assertRaisesRegex(ValueError,'missing required'):lint(s)
        s=spec();s['geometry'].pop('radius')
        with self.assertRaisesRegex(ValueError,'missing geometry'):lint(s)

    def test_independent_toggle_energy(self):
        # Differentiate scalar strain energy, then independently bisect its force.
        for element in ['truss','frame']:
            s=spec('shallow-toggle-'+element)
            def energy(v):
                a,h=2.,.1;l0=math.hypot(a,h);l=math.hypot(a,h-v)
                axial=2e6*(l-l0)**2/l0
                beta=math.atan((h-v)/a)-math.atan(h/a)
                return axial+(3*2e4*beta**2/l0 if element=='frame' else 0.)
            def derivative(v):
                d=1e-5
                return (energy(v-2*d)-8*energy(v-d)+8*energy(v+d)-energy(v+2*d))/(12*d)
            lo,hi=0.,.04
            for _ in range(60):
                mid=(lo+hi)/2
                if derivative(mid)<50:lo=mid
                else:hi=mid
            self.assertLess(abs((lo+hi)/2-s['references'][1]['value']),s['references'][1]['precision'])


if __name__ == '__main__':unittest.main()
