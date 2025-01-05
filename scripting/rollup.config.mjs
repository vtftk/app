
import path from 'node:path';
import { globSync } from 'glob';
import { dts } from "rollup-plugin-dts";
import { fileURLToPath } from 'node:url';
import terser from '@rollup/plugin-terser';
import typescript from '@rollup/plugin-typescript';
import { nodeResolve } from '@rollup/plugin-node-resolve';

import fixGlobalFile from './fix-global-file.mjs';

/**
 * Helper to get all the .test.ts files for compiling tests
 * 
 * @returns The list of test input files
 */
function getTestFiles() {
    return Object.fromEntries(globSync('src/**/*.test.ts').map(file => [
        // This remove `src/` as well as the file extension from each
        // file, so e.g. src/nested/foo.js becomes nested/foo
        path.relative(
            'src',
            file.slice(0, file.length - path.extname(file).length)
        ),
        // This expands the relative paths to absolute paths, so e.g.
        // src/nested/foo becomes /project/src/nested/foo.js
        fileURLToPath(new URL(file, import.meta.url))
    ]));
}



export default [
    // Main runtime bundle
    {
        input: 'src/index.ts',
        output: {
            file: 'dist/runtime.js',
            format: 'es'
        },
        plugins: [
            // Resolve imported modules
            nodeResolve(),
            // Parse and compile typescript
            typescript(),
            // Minify output
            terser({
                format: {
                    // Deno extensions can only use ASCII characters
                    ascii_only: true
                }
            })
        ]
    },

    // Compiled test javascript
    {
        input: getTestFiles(),
        output: {
            dir: 'dist',
            format: 'es'
        },
        plugins: [typescript()],
        external: ['jest'],
    },

    // Bundle typescript definitions
    {
        input: "dist/types/index.d.ts",
        output: [{ file: "dist/runtime.d.ts", format: "es" }],
        plugins: [
            dts({
                respectExternal: true,
            }),
            fixGlobalFile(),
        ],
    },
];