#include <sys/types.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

int main(int argc, char** argv) {
    if (argc != 2) {
        printf("fork: expected 1 parameter, given %d\n", argc - 1);
        return 1;
    }
    
    int wait_time = atoi(argv[1]);
    if (fork()) {
        printf("Main Process: sleeping %d seconds\n", wait_time);
        sleep(wait_time);
    } else {
        printf("Child Process: sleeping %d seconds\n", wait_time);
        sleep(wait_time);
    }

    return 0;
}