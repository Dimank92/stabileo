### Fixed
#### PRO DXF of the envelope draws the envelope; no deformed shape for an envelope or SRSS set (2026-10-21)

The DXF integrated the envelope's end forces as one load state: each end's largest force, from
whichever combination gives it, with the first combination's member loads. On a portal whose
wind combination is listed before the dead load the beam was drawn and labelled at 320.59 kN·m
against an envelope peak of 49.16. It now draws the envelope's own positive and negative station
curves, labelled and scaled from the envelope, as the figures do (#271); the same holds when the
envelope is the set on screen. An envelope (each value from its own combination) and an SRSS or
ABS set (magnitudes without a sign) are no deflected state: neither writes a deformed shape any
more, and their reactions are written as values without an arrow. The RESULTS layer says what
was left out (W4-01, W4-02).
