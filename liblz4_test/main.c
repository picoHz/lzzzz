#include <stdio.h>
#include <string.h>
#include "lz4frame.h"

int main(int argc, const char * argv[]) {
    LZ4F_preferences_t pref = { 0 };
    pref.compressionLevel = 0;
    
    const char *src = "Hello";
    char dst[256];
    LZ4F_compressFrame(dst, sizeof(dst), src, strlen(src), &pref);
    
    // insert code here...
    printf("Hello, World!\n");
    return 0;
}
