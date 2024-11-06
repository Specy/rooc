import {execSync} from 'child_process';
import fs from "fs/promises"



async function init(){
    console.log("Starting build...")
    await fs.unlink("./src/pkg/rooc_bg.wasm.d.ts").catch(() => console.warn("rooc_bg.wasm.d.ts not found"));
    execSync('tsc', {stdio: 'inherit'});
    await fs.cp("./src/pkg", "./dist/pkg", { recursive: true });
    await fs.unlink("./dist/pkg/package.json").catch(() => console.warn("package.json not found"));
    await fs.unlink("./dist/pkg/README.md").catch(() => console.warn("README.md not found"));
    await fs.unlink("./dist/pkg/.gitignore").catch(() => console.warn(".gitignore not found"));
    console.log("Build complete")
    
}

init()