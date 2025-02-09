# 1-49-puzzle-solver

Solver for the puzzle https://cruxpuzzles.co.uk/products/1-49-new

![](./docs/image.png)

In code, the pieces are represent using hex (so 10 is `a`, 11 is `b` etc).

The `0` piece is the "teeth" of the board edge.

It is assumed that the pieces can be rotated and flipped.

Note currently this returns the first solution, instead of all solutions (as finding all solutions can take a lot
longer).

# How to run

```bash
# Solve for 46
RUSTFLAGS=-Awarnings cargo run --bin solver -- -t 46
```

This prints the solution (if found) like below:

![img.png](docs/solution.png)

Your terminal needs to support true color to show the colors. To check, run `echo $COLORTERM` and see if output is
`truecolor`.

# Brief explanation of the algorithm

The core is [depth first search](https://en.wikipedia.org/wiki/Depth-first_search).

Each child node is created by placing an unused piece on the board.

To limit the search space, it keeps track of the next free coordinate, and aligns the current piece's top-left corner
with that coordinate, then computes whether the piece can be placed.
