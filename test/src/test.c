#include <trap.h>

unsigned short mem[] = {
	0x0, 0x0258, 0x4abc, 0x7fff, 0x8000, 0x8100, 0xabcd, 0xffff
};

unsigned  sh_ans[] = {
	0x0000fffd, 0x0000fff7, 0x0000ffdf, 0x0000ff7f, 0x0000fdff, 0x0000f7ff, 0x0000dfff, 0x00007fff
};

int main() {
	unsigned i;

	for(i = 0; i < LENGTH(mem); i ++) {
		mem[i] = ~(1 << (2 * i + 1));
		check(mem[i] == sh_ans[i]);
	}

	return 0;
}