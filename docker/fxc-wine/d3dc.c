/* Minimal fxc stand-in for cross-compiling gpui-fast-windows from Linux: runs under wine, uses
   d3dcompiler_47 (wine's, vkd3d-shader based). Understands only what the crate's build.rs passes:
   /T target /E entry /Fh header /Vn name /O3 file. Output: `const BYTE name[] = {...};` */
#define COBJMACROS
#include <windows.h>
#include <d3dcompiler.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
    const char *target = 0, *entry = 0, *out = 0, *name = "shader", *file = 0;
    for (int i = 1; i < argc; i++) {
        if (!strcmp(argv[i], "/T")) target = argv[++i];
        else if (!strcmp(argv[i], "/E")) entry = argv[++i];
        else if (!strcmp(argv[i], "/Fh")) out = argv[++i];
        else if (!strcmp(argv[i], "/Vn")) name = argv[++i];
        else if (argv[i][0] == '/' && argv[i][1] && strchr("OZWV", argv[i][1]) && strlen(argv[i]) <= 4) continue;
        else file = argv[i];
    }
    if (!target || !entry || !out || !file) { fprintf(stderr, "usage\n"); return 2; }
    WCHAR wfile[MAX_PATH * 2];
    MultiByteToWideChar(CP_UTF8, 0, file, -1, wfile, MAX_PATH * 2);
    ID3DBlob *code = 0, *err = 0;
    HRESULT hr = D3DCompileFromFile(wfile, 0, D3D_COMPILE_STANDARD_FILE_INCLUDE, entry, target,
                                    D3DCOMPILE_OPTIMIZATION_LEVEL3, 0, &code, &err);
    if (err) fwrite(ID3D10Blob_GetBufferPointer(err), 1, ID3D10Blob_GetBufferSize(err), stderr);
    if (FAILED(hr) || !code) { fprintf(stderr, "D3DCompileFromFile failed: 0x%08lx\n", hr); return 1; }
    const unsigned char *p = ID3D10Blob_GetBufferPointer(code);
    SIZE_T n = ID3D10Blob_GetBufferSize(code);
    FILE *f = fopen(out, "w");
    if (!f) { perror(out); return 1; }
    fprintf(f, "const BYTE %s[%lu] =\n{\n", name, (unsigned long)n);
    for (SIZE_T i = 0; i < n; i++) fprintf(f, "%s0x%02x,", i % 12 ? " " : "\n    ", p[i]);
    fprintf(f, "\n};\n");
    fclose(f);
    return 0;
}
