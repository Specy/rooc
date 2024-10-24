import {makeRoocFunction, PrimitiveKind} from "@specy/rooc";


const sqrt = makeRoocFunction({
    name: 'sqrt',
    description: 'Calculate the square root of a number',
    returnType: PrimitiveKind.Number,
    argTypes: [['of_num', PrimitiveKind.Number]],
    call: (num) => {
        return {type: "Number", value: Math.sqrt(num.value)}
    }
})



export function roocJsStd() {
    return [
        sqrt
    ]
}

