### Changed
#### Realistic example models: stadium live load, stay pretension, and offshore platform flare deck sag (#372)

The PRO catalogue example models are brought into line with what they claim to teach:
- `la-bombonera`: replaced the token 3 kN node loads with realistic CIRSOC 101 grandstand live loads (75 kN per node), bringing the resultant above 5 000 kN.
- `cable-stayed-bridge` and `cable-stayed-bridge-small`: updated stays to `behaviour: "cable"` with 200 kN pretension, eliminating the large dead-load deflection.
- `offshore-platform`: stiffened topside deck under the flare boom to prevent excessive differential sag relative to the jacket corner legs.
- `loadFixture`: preserves member `behaviour` and `cablePretension` when instantiating fixture models.
