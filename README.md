A 6502 emulator. I might emulate the entire nes later.

There are oher ways to write emulators. This is my first and I hoped to keep i simple. But no too simple it won't be cool.
The easiest is instruction stepped and instruction ticked, meaning that you provide functions for each instrucion and memory access, therefore the cpu state and registers can only be worth any value at the end of every instruction fetch-decode-exec cycle. 

The other style is instruction stepped and cycle ticked. Here, an instruction is a unit. You step in insructions, but every cycle is accounted for in each step. You cannot tell the CPU to only execute clock cycle and stop; Instructions are atomic. But the state of te system is . In such an emulator, there's a cycle/tick  funcion that is called in each exec/mem access function, updating the cycle as needed.

The third, which is the most complex, correct, and rewarding, is the cycle-stepped and cycle tickd. A step is a tick. A tick is a cycle. The tick is not some foreign function called. In such an emulator, you would need helper functions, of course, but the CPU step is one large function accounting for every possiple decoded innstruction in eac step. You can stop the cpu at the end of any cycle. 