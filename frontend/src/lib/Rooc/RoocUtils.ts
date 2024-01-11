import type { PossibleCompletionToken, SerializedPrimitiveKind } from "@specy/rooc"

export function getFormattedRoocType(type: SerializedPrimitiveKind) {
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
        return `${entry.name}(${entry.parameters.map(v => `${v.name}: ${getFormattedRoocType(v.value)}`).join(", ")}): ${getFormattedRoocType(entry.returnType)}`
    } else if (entry.type === "RuntimeBlockFunction"){
        return `${entry.name}{ }`
    } else if(entry.type === "RuntimeBlockScopedFunction"){
        return `${entry.name}(){ }`
    }
    return ""
}