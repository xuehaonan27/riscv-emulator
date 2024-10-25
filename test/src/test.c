#include <trap.h>

int ackermann(int m, int n) {
	if(m == 0) {
		return n + 1;
	} else {
		if(n == 0) {
			return ackermann(m - 1, 1);
		} else {
			return ackermann(m - 1, ackermann(m, n - 1));
		}
	}
}

int main() {
	int m, n, result;
    m = 1;
    n = 2;

	result = ackermann(m, n);
    check(result == 4);

	return 0;
}