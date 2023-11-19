# Tjangas

Tjangas is one of my most honest attempts at a chess engine so far. From code organization to profiling, this engine is a mile above my previous ones. Of course, I have only partially implemented move generation and will only focus on search after a desired NPS goal has been reached.

## Code Organization
The project contains a `types` and `engine` package. Since I plan on reserving `engine` for move-generation, this might be renamed to `movegen` or similar. 


## Profiling
All profiling is currently single-threaded to reduce complexity and easily identify hot spots.

As can be seen from the flamegraph, `Board::danger` is responsible for more than 70% of runtime, making it a flaming hot spot. This is mainly because of the sheer amount of bit-manipulation instructions necessary to find all slider moves in parallel. After ensuring my move-generator is 100% legal, I'll consider some alternatives to calculating this mask and whether or not I can generate legal king moves without it, since that is the only purpose it is serving as of right now. 