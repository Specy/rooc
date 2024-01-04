import { RoocParser} from "./dist/index.js"


const parser = new RoocParser(`
min 1
 10 + 20

`)


console.log(parser.compile())