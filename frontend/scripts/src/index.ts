import {parse} from 'csv-parse/browser/esm/sync'
import type {Options} from 'csv-parse/sync'
import {alg, Edge, Graph} from '@dagrejs/graphlib'

export namespace Rooc {
    //leave this as is, it will be used to patch the dts file
    export function start_ROOC() {
    }

    const defaultConfig = {
        cast: true,
        skip_empty_lines: true,
        trim: true,
    } satisfies Options

    type SerializedGraphEdge = {
        from: string,
        to: string,
        weight?: number
    }


    type SerializedGraphNode = {
        name: string,
        edges: { [key: string]: SerializedGraphEdge }
    }


    type SerializedGraph = {
        vertices: SerializedGraphNode[]
    }


    type PossibleValues = string | number | boolean | null

    /**
     * Parse a CSV table into a table of values
     * @param input
     * @param delimiter
     * @param options
     */
    export function parseCsvTable(input: string, delimiter: string = ",", options?: Options): PossibleValues[][] {
        return parse(input, {...defaultConfig, delimiter, ...options})
    }

    /**
     * Parse a CSV table into a table of objects, it must have a header row
     * @param input
     * @param delimiter
     * @param options
     */
    export function parseCsvObject<T>(input: string, delimiter: string = ",", options?: Options): T[] {
        return parse(input, {...defaultConfig, delimiter, columns: true, ...options})
    }

    export function fromSerializedGraph(serializedGraph: SerializedGraph): Graph {
        const graph = new Graph()
        serializedGraph.vertices.forEach((vertex) => {
            graph.setNode(vertex.name, vertex)
            Object.entries(vertex.edges).forEach(([key, edge]) => {
                graph.setEdge(edge.from, edge.to, {label: key, weight: edge.weight})
            })
        })
        return graph
    }

    export function toSerializedGraph(graph: Graph): SerializedGraph {
        const vertices: SerializedGraphNode[] = graph.nodes().map((node) => {
            const edges = (graph.nodeEdges(node) as Edge[]).reduce((acc, edge) => {
                const e = graph.edge(edge)
                acc[edge.name] = {from: edge.v, to: edge.w, weight: e.weight}
                return acc
            }, {})
            return {name: node, edges}
        })
        return {vertices}
    }

    export const RoocGraph = Graph

    export const GraphAlgorithms = alg

    //leave this as is, it will be used to patch the dts file
    export function end_ROOC() {
    }
}
