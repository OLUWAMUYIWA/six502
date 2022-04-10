- check for `comeback`s


- an interesting way to check for overflow in (a + b = result) (a ^ b) & 0x80 == 0 && (a ^ result) & 0x80 == 0x80,