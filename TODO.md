Add WASM Coding
---------------

- decode opcode from wasm

  My approach was to match the main discriminant bits ((opcode & 0xF000) >> 12, opcode & 0x000F)
  then within each case use helper functions to extract the useful information e.g. x(opcode)
  (Register(((opcode & 0x0F00) >> 8) as u8)), addr(opcode) (opcode & 0x0FFF), ...

  The base cases could be sub-matched when that made sense e.g. the Fxcc class. 6xkk would be
  (0x6, _) => Instruction::LoadValue { dest: x(opcode), source: byte(opcode) }

- make bot with a single command in wat
- replace wat by rust code
- insert correct code from template
- create new bots
- add external functions that the bot's code can access
- add gas

UI
--

- fix bug in visualizing squads

Code quality
------------

- review

Contente
--------

- create many levels
- create many bots
