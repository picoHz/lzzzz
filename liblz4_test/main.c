#include <stdio.h>
#include <string.h>
#include <memory.h>
#include "lz4frame.h"

int main(int argc, const char * argv[]) {
    char src[1023];
    char dst[65544];
    LZ4F_preferences_t pref;

    memset(&pref, 0, sizeof(pref));
    pref.compressionLevel = -47366883;
    for (int i = 0; i < sizeof(src); ++i) {
        src[i] = i;
    }

    LZ4F_compressFrame(dst, sizeof(dst), src, sizeof(src), &pref);
    return 0;
}
