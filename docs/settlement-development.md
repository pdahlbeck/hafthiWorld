# Settlement development after the renderer and anatomy fixes

Issue #1 calls for a shared household economy and the ability to observe the same
world in strategy and first person. Keep each slice small enough to test and play.

## Delivered: repeat coastal production

U authorizes a later spring/summer round only after fleece has regrown and every
item from the old batch has reached its terminal state. Actual labor, physical
carrying, payment and food consumption still perform the work. The regression
runs three material-balanced rounds, two worker entities across camera switches,
save/restore during carrying and legacy saves.

## Delivered: another farmstead

The fixed second building requires actual timber transport and assembly before
its resident can be invited. Household 2 has separately owned food/fuel, separate
demand clocks and its own provider; the original founder and optional coastal
helper retain their roles. Both homes compete for the same bounded inventory.
Save/load, three-resident animation, collisions, camera switches and races for
one item are tested. See [the second-household guide](second-household.md).

## Delivered: forestry and timber planning

X orders real planting work, growth pauses in winter, and F harvests a new crop
when mature. New units get their own ledger records, preserving all spent wood.
M chooses Balanced, Building first or Winter reserves. Real transport still
allocates every unit; priority changes never reclaim committed deliveries.
See [controls and validation](forestry-and-timber.md).

## Further slices

- Independent fields and per-household work allocation.
- More flexible building placement and navigation.
- Additional historically researched crafts and limited Baltic trade goods.
- Improved sheep/clothing art and household/appearance variants.

Not included now: unlimited population, unrestricted building,
starvation/death, combat or a complete economy. Yields and time intervals are
design choices, not archaeological claims.
