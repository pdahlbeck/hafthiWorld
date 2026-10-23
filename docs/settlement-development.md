# Settlement development after the renderer and anatomy fixes

Issue #1 calls for a shared household economy, not several independent camera
scenes. Keep each change small enough to test and play before expanding again.

## Delivered in this slice: repeat coastal production

A completed coastal batch no longer exhausts fishing, wool, salt and shipment
forever. U authorizes a later spring/summer round only after fleece has regrown
and every item from the old batch has reached its terminal state. Actual labor,
physical carrying, payment and food consumption still perform the work. Existing
orders, saved time, two-worker ownership and entity identities persist.

The full regression runs three material-balanced rounds (12 fish meals, six
salting batches and three exported wool batches). A runtime test runs two rounds
with the same two worker entities while repeatedly switching cameras. Save
checks restore carried wool, the growth clocks and cumulative round history;
legacy saves accept defaults. The original one-round tests remain applicable.

## Next scoped expansion: another farmstead

Add a second actual household and building site funded by real materials, then
allow a new household to settle. Define per-household storage, food/fuel demand,
residents and work assignment before increasing the hard-coded worker count.
Navigation must keep construction footprints and entrances traversable in both
views. Test two households competing for limited goods, winter demand, save/load
and moving real building materials; do not duplicate an inventory to create a
new settlement.

## Further slices

- Renewable managed timber and real firewood demand beyond the first winters.
- Additional historically researched crafts and limited Baltic trade goods.
- More flexible field/building placement and worker scheduling.
- Improved sheep and clothing art, then household/appearance variants.

Not included now: a second household, population growth, starvation/death,
renewable timber, unrestricted construction, combat or a complete economy.
Gameplay yields and growth intervals remain design choices, not historical data.
