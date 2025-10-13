#include <stdio.h>
#include <unistd.h> 
#include <stdlib.h>

int main() {
    int width = 50;
    int pos = 0;    
    int direction = 1; 

    while (1) {
        system("clear"); 
        for (int i = 0; i < pos; i++)
            printf(" "); 

        printf("☭\n"); 

        usleep(100000); 

        pos += direction;
        if (pos == width || pos == 0)
            direction = -direction; 
    }

    return 0;
}

