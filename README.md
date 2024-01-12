## A Chip8 emulator written in Rust!

### Setup
This assumes you have `cargo` installed.
Also, you will need to install `sdl2` - see [here](https://github.com/Rust-SDL2/rust-sdl2) for install instructions.

Usage:
`cargo run /path/to/file`


### Download ya some games!
[Chip8 Games](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)


### Implementation notes:

I primarily used the "Cowgod's Chip 8 Specification". I found this specification to be quite good but there were a few things that I misinterpreted that cost me a bit of time and, for those who might be reading this in the future, I will mention those things below.

1. 
```
2nnn - CALL addr
Call subroutine at nnn.

The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
```
I found this to be a little confusing as I think what actually needs to happen is that you put the current PC on top of the stack _and then_ increment the stack pointer. Finally, set the PC to nnn. The way this reads is that you would _first_ increment the stack pointer which is not right.

2.
```
00EE - RET
Return from a subroutine.

The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
```
Similar to point 1, I misintrepreted the order here. What should happen is that you decrement the stack pointer _and then_ set the program counter to the top of the stack.

Both points 1 and 2 make when you think about what a `CALL` and `RETURN` statement should do given a program counter and a stack but I still messed this up the first time.

In general, be careful when adding `u8`s (ie make sure you account for potential overflows).


Sources:
- [Cowgods' Specification](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Reference implementation](https://github.com/starrhorne/chip8-rust)