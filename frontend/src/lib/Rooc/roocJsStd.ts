
/*
import {makeRoocFunction, PrimitiveKind} from "@specy/rooc";
const sqrt = makeRoocFunction({
    name: 'sqrt',
    description: 'Calculate the square root of a number',
    returns: PrimitiveKind.Number,
    parameters: [['of_num', PrimitiveKind.Number]],
    call: (num) => {
        return {type: "Number", value: Math.sqrt(num.value)}
    }
})
*/


export function roocJsStd() {
    return [
        //sqrt
    ]
}

