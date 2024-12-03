One probably has to start over or make a major refactoring, copying some data structures and functions: red/blue, invert, team... all these has to change.

If we were to attempt to build a world with these bots, here is a step by step guide:

1 - increase the board size, with many "screens" (let us call them region), say 100x100 screens of 64x64
2 - make a mini-map for now
3 - later we can add zoom + scroll
4 - make script for initial distribution of resources
5 - add the notion of players to the game (using ethereum addresses)
6 - add materials to each player
7 - add money to each player
8 - add ownership to the regions (each region belongs to an ethereum address)
9 - each player can sell/buy regions
10 - when units die, a part of their resourses get spread to regions (re-minted in proportion to owners). Some part goes to the floor and can be picked
11 - robots can donate from inventory to ehtereum address
12 - owners can replace whatever is in their regions by something that fits their material budget. Whatever was in their region before goes to their materials. They need to pay some sort to tax.
