cd compiler; cargo build; cd ..
cd assembler; cargo build; cd ..
cd simulator; cargo build; cd ..
./compiler/target/debug/kcc tests/test.c > test.asm
./assembler/target/debug/kasm test.asm > test.bin
./simulator/target/debug/ksim test.bin > test.txt
diff test.txt tests/expect.txt

