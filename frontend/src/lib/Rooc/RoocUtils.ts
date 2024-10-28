import type { PossibleCompletionToken, SerializedPrimitiveKind, RoocFunction } from "@specy/rooc"
import type {NamedParameter, RuntimeFunction} from "@specy/rooc/runtime";

export function roocFunctionToRuntimeFunction(f: RoocFunction): RuntimeFunction<NamedParameter[], SerializedPrimitiveKind> {
    return {
        name: f.name,
        description: f.description,
        type: "RuntimeFunction",
        parameters: f.parameters.map(([k, v]) => ({name: k, value: v})),
        returns: typeof f.returns === 'function' ? {type: "Any"} : f.returns
    } satisfies RuntimeFunction<NamedParameter[], SerializedPrimitiveKind>
}


export function getFormattedRoocType(type: SerializedPrimitiveKind): string {
	if (type.type === 'Tuple') {
		return `(${type.value.map(getFormattedRoocType).join(', ')})`
	} else if (type.type === "Iterable") {
		return `${getFormattedRoocType(type.value)}[]`
	} else {
		return type.type
	}
}

export function createRoocFunctionSignature(entry: PossibleCompletionToken){
    if(entry.type === "RuntimeFunction"){
        return `${entry.name}(${entry.parameters.map(v => `${v.name}: ${getFormattedRoocType(v.value)}`).join(", ")}): ${getFormattedRoocType(entry.returns)}`
    } else if (entry.type === "RuntimeBlockFunction"){
        return `${entry.name}{ }`
    } else if(entry.type === "RuntimeBlockScopedFunction"){
        return `${entry.name}(){ }`
    }
    return ""
}