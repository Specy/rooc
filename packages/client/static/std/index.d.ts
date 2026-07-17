

declare class CsvError extends Error {
	readonly code: CsvErrorCode;
	[key: string]: any;
	constructor(code: CsvErrorCode, message: string | string[], options?: Options, ...contexts: any[]);
}
declare class Graph {
	constructor(options?: GraphOptions);
	/**
	 * Sets the default node label. This label will be assigned as default label
	 * in case if no label was specified while setting a node.
	 * Complexity: O(1).
	 *
	 * @argument label - default node label.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setDefaultNodeLabel(label: any): Graph;
	/**
	 * Sets the default node label factory function. This function will be invoked
	 * each time when setting a node with no label specified and returned value
	 * will be used as a label for node.
	 * Complexity: O(1).
	 *
	 * @argument labelFn - default node label factory function.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setDefaultNodeLabel(labelFn: (v: string) => any): Graph;
	/**
	 * Creates or updates the value for the node v in the graph. If label is supplied
	 * it is set as the value for the node. If label is not supplied and the node was
	 * created by this call then the default node label will be assigned.
	 * Complexity: O(1).
	 *
	 * @argument name - node name.
	 * @argument label - value to set for node.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setNode(name: string, label?: any): Graph;
	/**
	 * Invokes setNode method for each node in names list.
	 * Complexity: O(|names|).
	 *
	 * @argument names - list of nodes names to be set.
	 * @argument label - value to set for each node in list.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setNodes(names: string[], label?: any): Graph;
	/**
	 * Sets node p as a parent for node v if it is defined, or removes the
	 * parent for v if p is undefined. Method throws an exception in case of
	 * invoking it in context of noncompound graph.
	 * Average-case complexity: O(1).
	 *
	 * @argument v - node to be child for p.
	 * @argument p - node to be parent for v.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setParent(v: string, p?: string): Graph;
	/**
	 * Gets parent node for node v.
	 * Complexity: O(1).
	 *
	 * @argument v - node to get parent of.
	 * @returns parent node name or void if v has no parent.
	 */
	parent(v: string): string | void;
	/**
	 * Gets list of direct children of node v.
	 * Complexity: O(1).
	 *
	 * @argument v - node to get children of.
	 * @returns children nodes names list.
	 */
	children(v: string): string[];
	/**
	 * Creates new graph with nodes filtered via filter. Edges incident to rejected node
	 * are also removed. In case of compound graph, if parent is rejected by filter,
	 * than all its children are rejected too.
	 * Average-case complexity: O(|E|+|V|).
	 *
	 * @argument filter - filtration function detecting whether the node should stay or not.
	 * @returns new graph made from current and nodes filtered.
	 */
	filterNodes(filter: (v: string) => boolean): Graph;
	/**
	 * Sets the default edge label. This label will be assigned as default label
	 * in case if no label was specified while setting an edge.
	 * Complexity: O(1).
	 *
	 * @argument label - default edge label.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setDefaultEdgeLabel(label: any): Graph;
	/**
	 * Sets the default edge label factory function. This function will be invoked
	 * each time when setting an edge with no label specified and returned value
	 * will be used as a label for edge.
	 * Complexity: O(1).
	 *
	 * @argument labelFn - default edge label factory function.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setDefaultEdgeLabel(labelFn: (v: string) => any): Graph;
	/**
	 * Establish an edges path over the nodes in nodes list. If some edge is already
	 * exists, it will update its label, otherwise it will create an edge between pair
	 * of nodes with label provided or default label if no label provided.
	 * Complexity: O(|nodes|).
	 *
	 * @argument nodes - list of nodes to be connected in series.
	 * @argument label - value to set for each edge between pairs of nodes.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setPath(nodes: string[], label?: any): Graph;
	/**
	 * Detects whether graph has a node with specified name or not.

	 *
	 * @argument name - name of the node.
	 * @returns true if graph has node with specified name, false - otherwise.
	 */
	hasNode(name: string): boolean;
	/**
	 * Remove the node with the name from the graph or do nothing if the node is not in
	 * the graph. If the node was removed this function also removes any incident
	 * edges.
	 * Complexity: O(1).
	 *
	 * @argument name - name of the node.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	removeNode(name: string): Graph;
	/**
	 * Gets all nodes of the graph. Note, the in case of compound graph subnodes are
	 * not included in list.
	 * Complexity: O(1).
	 *
	 * @returns list of graph nodes.
	 */
	nodes(): string[];
	/**
	 * Gets the label of node with specified name.
	 * Complexity: O(|V|).
	 *
	 * @returns label value of the node.
	 */
	node(name: string): any;
	/**
	 * Creates or updates the label for the edge (v, w) with the optionally supplied
	 * name. If label is supplied it is set as the value for the edge. If label is not
	 * supplied and the edge was created by this call then the default edge label will
	 * be assigned. The name parameter is only useful with multigraphs.
	 * Complexity: O(1).
	 *
	 * @argument v - edge source node.
	 * @argument w - edge sink node.
	 * @argument label - value to associate with the edge.
	 * @argument name - unique name of the edge in order to identify it in multigraph.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setEdge(v: string, w: string, label?: any, name?: string): Graph;
	/**
	 * Creates or updates the label for the specified edge. If label is supplied it is
	 * set as the value for the edge. If label is not supplied and the edge was created
	 * by this call then the default edge label will be assigned. The name parameter is
	 * only useful with multigraphs.
	 * Complexity: O(1).
	 *
	 * @argument edge - edge descriptor.
	 * @argument label - value to associate with the edge.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setEdge(edge: Edge, label?: any): Graph;
	/**
	 * Gets edges of the graph. In case of compound graph subgraphs are not considered.
	 * Complexity: O(|E|).
	 *
	 * @return graph edges list.
	 */
	edges(): Edge[];
	/**
	 * Gets the label for the specified edge.
	 * Complexity: O(1).
	 *
	 * @argument v - edge source node.
	 * @argument w - edge sink node.
	 * @argument name - name of the edge (actual for multigraph).
	 * @returns value associated with specified edge.
	 */
	edge(v: string, w: string, name?: string): any;
	/**
	 * Gets the label for the specified edge.
	 * Complexity: O(1).
	 *
	 * @argument edge - edge descriptor.
	 * @returns value associated with specified edge.
	 */
	edge(e: Edge): any;
	/**
	 * Gets the label for the specified edge and converts it to an object.
	 * Complexity: O(1).
	 *
	 * @argument v - edge source node.
	 * @argument w - edge sink node.
	 * @argument name - name of the edge (actual for multigraph).
	 * @returns value associated with specified edge.
	 */
	edgeAsObj(v: string, w: string, name?: string): Object;
	/**
	 * Gets the label for the specified edge and converts it to an object.
	 * Complexity: O(1).
	 *
	 * @argument edge - edge descriptor.
	 * @returns value associated with specified edge.
	 */
	edgeAsObj(e: Edge): Object;
	/**
	 * Detects whether the graph contains specified edge or not. No subgraphs are considered.
	 * Complexity: O(1).
	 *
	 * @argument v - edge source node.
	 * @argument w - edge sink node.
	 * @argument name - name of the edge (actual for multigraph).
	 * @returns whether the graph contains the specified edge or not.
	 */
	hasEdge(v: string, w: string, name?: string): boolean;
	/**
	 * Detects whether the graph contains specified edge or not. No subgraphs are considered.
	 * Complexity: O(1).
	 *
	 * @argument edge - edge descriptor.
	 * @returns whether the graph contains the specified edge or not.
	 */
	hasEdge(edge: Edge): boolean;
	/**
	 * Removes the specified edge from the graph. No subgraphs are considered.
	 * Complexity: O(1).
	 *
	 * @argument edge - edge descriptor.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	removeEdge(edge: Edge): Graph;
	/**
	 * Removes the specified edge from the graph. No subgraphs are considered.
	 * Complexity: O(1).
	 *
	 * @argument v - edge source node.
	 * @argument w - edge sink node.
	 * @argument name - name of the edge (actual for multigraph).
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	removeEdge(v: string, w: string, name?: string): Graph;
	/**
	 * Return all edges that point to the node v. Optionally filters those edges down to just those
	 * coming from node u. Behavior is undefined for undirected graphs - use nodeEdges instead.
	 * Complexity: O(|E|).
	 *
	 * @argument v - edge sink node.
	 * @argument w - edge source node.
	 * @returns edges descriptors list if v is in the graph, or undefined otherwise.
	 */
	inEdges(v: string, w?: string): void | Edge[];
	/**
	 * Return all edges that are pointed at by node v. Optionally filters those edges down to just
	 * those point to w. Behavior is undefined for undirected graphs - use nodeEdges instead.
	 * Complexity: O(|E|).
	 *
	 * @argument v - edge source node.
	 * @argument w - edge sink node.
	 * @returns edges descriptors list if v is in the graph, or undefined otherwise.
	 */
	outEdges(v: string, w?: string): void | Edge[];
	/**
	 * Returns all edges to or from node v regardless of direction. Optionally filters those edges
	 * down to just those between nodes v and w regardless of direction.
	 * Complexity: O(|E|).
	 *
	 * @argument v - edge adjacent node.
	 * @argument w - edge adjacent node.
	 * @returns edges descriptors list if v is in the graph, or undefined otherwise.
	 */
	nodeEdges(v: string, w?: string): void | Edge[];
	/**
	 * Return all nodes that are predecessors of the specified node or undefined if node v is not in
	 * the graph. Behavior is undefined for undirected graphs - use neighbors instead.
	 * Complexity: O(|V|).
	 *
	 * @argument v - node identifier.
	 * @returns node identifiers list or undefined if v is not in the graph.
	 */
	predecessors(v: string): void | string[];
	/**
	 * Return all nodes that are successors of the specified node or undefined if node v is not in
	 * the graph. Behavior is undefined for undirected graphs - use neighbors instead.
	 * Complexity: O(|V|).
	 *
	 * @argument v - node identifier.
	 * @returns node identifiers list or undefined if v is not in the graph.
	 */
	successors(v: string): void | string[];
	/**
	 * Return all nodes that are predecessors or successors of the specified node or undefined if
	 * node v is not in the graph.
	 * Complexity: O(|V|).
	 *
	 * @argument v - node identifier.
	 * @returns node identifiers list or undefined if v is not in the graph.
	 */
	neighbors(v: string): void | string[];
	/**
	 * Whether graph was created with 'directed' flag set to true or not.
	 *
	 * @returns whether the graph edges have an orientation.
	 */
	isDirected(): boolean;
	/**
	 * Whether graph was created with 'multigraph' flag set to true or not.
	 *
	 * @returns whether the pair of nodes of the graph can have multiple edges.
	 */
	isMultigraph(): boolean;
	/**
	 * Whether graph was created with 'compound' flag set to true or not.
	 *
	 * @returns whether a node of the graph can have subnodes.
	 */
	isCompound(): boolean;
	/**
	 * Sets the label of the graph.
	 *
	 * @argument label - label value.
	 * @returns the graph, allowing this to be chained with other functions.
	 */
	setGraph(label: any): Graph;
	/**
	 * Gets the graph label.
	 *
	 * @returns currently assigned label for the graph or undefined if no label assigned.
	 */
	graph(): any;
	/**
	 * Gets the number of nodes in the graph.
	 * Complexity: O(1).
	 *
	 * @returns nodes count.
	 */
	nodeCount(): number;
	/**
	 * Gets the number of edges in the graph.
	 * Complexity: O(1).
	 *
	 * @returns edges count.
	 */
	edgeCount(): number;
	/**
	 * Gets list of nodes without in-edges.
	 * Complexity: O(|V|).
	 *
	 * @returns the graph source nodes.
	 */
	sources(): string[];
	/**
	 * Gets list of nodes without out-edges.
	 * Complexity: O(|V|).
	 *
	 * @returns the graph source nodes.
	 */
	sinks(): string[];
}
declare namespace alg {
	/**
	 * Finds all connected components in a graph and returns an array of these components.
	 * Each component is itself an array that contains the ids of nodes in the component.
	 * Complexity: O(|V|).
	 *
	 * @argument graph - graph to find components in.
	 * @returns array of nodes list representing components
	 */
	function components(graph: Graph): string[][];
	/**
	 * This function is an implementation of Dijkstra's algorithm which finds the shortest
	 * path from source to all other nodes in graph. This function returns a map of
	 * v -> { distance, predecessor }. The distance property holds the sum of the weights
	 * from source to v along the shortest path or Number.POSITIVE_INFINITY if there is no path
	 * from source. The predecessor property can be used to walk the individual elements of the
	 * path from source to v in reverse order.
	 * Complexity: O((|E| + |V|) * log |V|).
	 *
	 * @argument graph - graph where to search pathes.
	 * @argument source - node to start pathes from.
	 * @argument weightFn - function which takes edge e and returns the weight of it. If no weightFn
	 * is supplied then each edge is assumed to have a weight of 1. This function throws an
	 * Error if any of the traversed edges have a negative edge weight.
	 * @argument edgeFn - function which takes a node v and returns the ids of all edges incident to it
	 * for the purposes of shortest path traversal. By default this function uses the graph.outEdges.
	 * @returns shortest pathes map that starts from node source
	 */
	function dijkstra(graph: Graph, source: string, weightFn?: (e: Edge) => number, edgeFn?: (v: string) => Edge[]): {
		[node: string]: Path;
	};
	/**
	 * This function finds the shortest path from each node to every other reachable node in
	 * the graph. It is similar to alg.dijkstra, but instead of returning a single-source
	 * array, it returns a mapping of source -> alg.dijksta(g, source, weightFn, edgeFn).
	 * Complexity: O(|V| * (|E| + |V|) * log |V|).
	 *
	 * @argument graph - graph where to search pathes.
	 * @argument weightFn - function which takes edge e and returns the weight of it. If no weightFn
	 * is supplied then each edge is assumed to have a weight of 1. This function throws an
	 * Error if any of the traversed edges have a negative edge weight.
	 * @argument edgeFn - function which takes a node v and returns the ids of all edges incident to it
	 * for the purposes of shortest path traversal. By default this function uses the graph.outEdges.
	 * @returns shortest pathes map.
	 */
	function dijkstraAll(graph: Graph, weightFn?: (e: Edge) => number, edgeFn?: (v: string) => Edge[]): {
		[source: string]: {
			[node: string]: Path;
		};
	};
	/**
	 * Given a Graph, graph, this function returns all nodes that are part of a cycle. As there
	 * may be more than one cycle in a graph this function return an array of these cycles,
	 * where each cycle is itself represented by an array of ids for each node involved in
	 * that cycle. Method alg.isAcyclic is more efficient if you only need to determine whether a graph has a
	 * cycle or not.
	 * Complexity: O(|V| + |E|).
	 *
	 * @argument graph - graph where to search cycles.
	 * @returns cycles list.
	 */
	function findCycles(graph: Graph): string[][];
	/**
	 * Given a Graph, graph, this function returns true if the graph has no cycles and returns false if it
	 * does. This algorithm returns as soon as it detects the first cycle. You can use alg.findCycles
	 * to get the actual list of cycles in the graph.
	 *
	 * @argument graph - graph to detect whether it acyclic ot not.
	 * @returns whether graph contain cycles or not.
	 */
	function isAcyclic(graph: Graph): boolean;
	/**
	 * This function is an implementation of the Floyd-Warshall algorithm, which finds the
	 * shortest path from each node to every other reachable node in the graph. It is similar
	 * to alg.dijkstraAll, but it handles negative edge weights and is more efficient for some types
	 * of graphs. This function returns a map of source -> { target -> { distance, predecessor }.
	 * The distance property holds the sum of the weights from source to target along the shortest
	 * path of Number.POSITIVE_INFINITY if there is no path from source. The predecessor property
	 * can be used to walk the individual elements of the path from source to target in reverse
	 * order.
	 * Complexity: O(|V|^3).
	 *
	 * @argument graph - graph where to search pathes.
	 * @argument weightFn - function which takes edge e and returns the weight of it. If no weightFn
	 * is supplied then each edge is assumed to have a weight of 1. This function throws an
	 * Error if any of the traversed edges have a negative edge weight.
	 * @argument edgeFn - function which takes a node v and returns the ids of all edges incident to it
	 * for the purposes of shortest path traversal. By default this function uses the graph.outEdges.
	 * @returns shortest pathes map.
	 */
	function floydWarshall(graph: Graph, weightFn?: (e: Edge) => number, edgeFn?: (v: string) => Edge[]): {
		[source: string]: {
			[node: string]: Path;
		};
	};
	/**
	 * Prim's algorithm takes a connected undirected graph and generates a minimum spanning tree. This
	 * function returns the minimum spanning tree as an undirected graph. This algorithm is derived
	 * from the description in "Introduction to Algorithms", Third Edition, Cormen, et al., Pg 634.
	 * Complexity: O(|E| * log |V|);
	 *
	 * @argument graph - graph to generate a minimum spanning tree of.
	 * @argument weightFn - function which takes edge e and returns the weight of it. It throws an Error if
	 *           the graph is not connected.
	 * @returns minimum spanning tree of graph.
	 */
	function prim(graph: Graph, weightFn: (e: Edge) => number): Graph;
	/**
	 * This function is an implementation of Tarjan's algorithm which finds all strongly connected
	 * components in the directed graph g. Each strongly connected component is composed of nodes that
	 * can reach all other nodes in the component via directed edges. A strongly connected component
	 * can consist of a single node if that node cannot both reach and be reached by any other
	 * specific node in the graph. Components of more than one node are guaranteed to have at least
	 * one cycle.
	 * Complexity: O(|V| + |E|).
	 *
	 * @argument graph - graph to find all strongly connected components of.
	 * @return  an array of components. Each component is itself an array that contains
	 *          the ids of all nodes in the component.
	 */
	function tarjan(graph: Graph): string[][];
	/**
	 * Given a Graph graph this function applies topological sorting to it.
	 * If the graph has a cycle it is impossible to generate such a list and CycleException is thrown.
	 * Complexity: O(|V| + |E|).
	 *
	 * @argument graph - graph to apply topological sorting to.
	 * @returns an array of nodes such that for each edge u -> v, u appears before v in the array.
	 */
	function topsort(graph: Graph): string[];
	/**
	 * Performs pre-order depth first traversal on the input graph. If the graph is
	 * undirected then this algorithm will navigate using neighbors. If the graph
	 * is directed then this algorithm will navigate using successors.
	 *
	 * @argument graph - depth first traversal target.
	 * @argument vs - nodes list to traverse.
	 * @returns the nodes in the order they were visited as a list of their names.
	 */
	function preorder(graph: Graph, vs: string[]): string[];
	/**
	 * Performs post-order depth first traversal on the input graph. If the graph is
	 * undirected then this algorithm will navigate using neighbors. If the graph
	 * is directed then this algorithm will navigate using successors.
	 *
	 * @argument graph - depth first traversal target.
	 * @argument vs - nodes list to traverse.
	 * @returns the nodes in the order they were visited as a list of their names.
	 */
	function postorder(graph: Graph, vs: string[]): string[];
}
declare namespace Rooc {
	
	type SerializedGraphEdge = {
		from: string;
		to: string;
		weight?: number;
	};
	type SerializedGraphNode = {
		name: string;
		edges: {
			[key: string]: SerializedGraphEdge;
		};
	};
	type SerializedGraph = {
		vertices: SerializedGraphNode[];
	};
	type PossibleValues = string | number | boolean | null;
	/**
	 * Parse a CSV table into a table of values
	 * @param input
	 * @param delimiter
	 * @param options
	 */
	export function parseCsvTable(input: string, delimiter?: string, options?: Options): PossibleValues[][];
	/**
	 * Parse a CSV table into a table of objects, it must have a header row
	 * @param input
	 * @param delimiter
	 * @param options
	 */
	export function parseCsvObject<T>(input: string, delimiter?: string, options?: Options): T[];
	export function fromSerializedGraph(serializedGraph: SerializedGraph): Graph;
	export function toSerializedGraph(graph: Graph): SerializedGraph;
	export const RoocGraph: typeof Graph;
	export const GraphAlgorithms: typeof alg;
	
	
}
declare interface CastingContext {
	readonly column: number | string;
	readonly empty_lines: number;
	readonly error: CsvError;
	readonly header: boolean;
	readonly index: number;
	readonly quoting: boolean;
	readonly lines: number;
	readonly records: number;
	readonly invalid_field_length: number;
}
declare interface Edge {
	v: string;
	w: string;
	/** The name that uniquely identifies a multi-edge. */
	name?: string;
}
declare interface GraphOptions {
	directed?: boolean; // default: true.
	multigraph?: boolean; // default: false.
	compound?: boolean; // default: false.
}
/*
Note, could not `extends stream.TransformOptions` because encoding can be
BufferEncoding and undefined as well as null which is not defined in the
extended type.
*/
declare interface Options {
	/**
	 * If true, the parser will attempt to convert read data types to native types.
	 * @deprecated Use {@link cast}
	 */
	auto_parse?: boolean | CastingFunction;
	autoParse?: boolean | CastingFunction;
	/**
	 * If true, the parser will attempt to convert read data types to dates. It requires the "auto_parse" option.
	 * @deprecated Use {@link cast_date}
	 */
	auto_parse_date?: boolean | CastingDateFunction;
	autoParseDate?: boolean | CastingDateFunction;
	/**
	 * If true, detect and exclude the byte order mark (BOM) from the CSV input if present.
	 */
	bom?: boolean;
	/**
	 * If true, the parser will attempt to convert input string to native types.
	 * If a function, receive the value as first argument, a context as second argument and return a new value. More information about the context properties is available below.
	 */
	cast?: boolean | CastingFunction;
	/**
	 * If true, the parser will attempt to convert input string to dates.
	 * If a function, receive the value as argument and return a new value. It requires the "auto_parse" option. Be careful, it relies on Date.parse.
	 */
	cast_date?: boolean | CastingDateFunction;
	castDate?: boolean | CastingDateFunction;
	/**
	 * List of fields as an array,
	 * a user defined callback accepting the first line and returning the column names or true if autodiscovered in the first CSV line,
	 * default to null,
	 * affect the result data set in the sense that records will be objects instead of arrays.
	 */
	columns?: ColumnOption[] | boolean | ((record: any) => ColumnOption[]);
	/**
	 * Convert values into an array of values when columns are activated and
	 * when multiple columns of the same name are found.
	 */
	group_columns_by_name?: boolean;
	groupColumnsByName?: boolean;
	/**
	 * Treat all the characters after this one as a comment, default to '' (disabled).
	 */
	comment?: string;
	/**
	 * Restrict the definition of comments to a full line. Comment characters
	 * defined in the middle of the line are not interpreted as such. The
	 * option require the activation of comments.
	 */
	comment_no_infix?: boolean;
	/**
	 * Set the field delimiter. One character only, defaults to comma.
	 */
	delimiter?: string | string[] | Buffer;
	/**
	 * Set the source and destination encoding, a value of `null` returns buffer instead of strings.
	 */
	encoding?: BufferEncoding | undefined;
	/**
	 * Set the escape character, one character only, defaults to double quotes.
	 */
	escape?: string | null | false | Buffer;
	/**
	 * Start handling records from the requested number of records.
	 */
	from?: number;
	/**
	 * Start handling records from the requested line number.
	 */
	from_line?: number;
	fromLine?: number;
	/**
	 * Don't interpret delimiters as such in the last field according to the number of fields calculated from the number of columns, the option require the presence of the `column` option when `true`.
	 */
	ignore_last_delimiters?: boolean | number;
	/**
	 * Generate two properties `info` and `record` where `info` is a snapshot of the info object at the time the record was created and `record` is the parsed array or object.
	 */
	info?: boolean;
	/**
	 * If true, ignore whitespace immediately following the delimiter (i.e. left-trim all fields), defaults to false.
	 * Does not remove whitespace in a quoted field.
	 */
	ltrim?: boolean;
	/**
	 * Maximum numer of characters to be contained in the field and line buffers before an exception is raised,
	 * used to guard against a wrong delimiter or record_delimiter,
	 * default to 128000 characters.
	 */
	max_record_size?: number;
	maxRecordSize?: number;
	/**
	 * Name of header-record title to name objects by.
	 */
	objname?: string;
	/**
	 * Alter and filter records by executing a user defined function.
	 */
	on_record?: (record: any, context: CastingContext) => any;
	onRecord?: (record: any, context: CastingContext) => any;
	/**
	 * Optional character surrounding a field, one character only, defaults to double quotes.
	 */
	quote?: string | boolean | Buffer | null;
	/**
	 * Generate two properties raw and row where raw is the original CSV row content and row is the parsed array or object.
	 */
	raw?: boolean;
	/**
	 * Discard inconsistent columns count, default to false.
	 */
	relax_column_count?: boolean;
	relaxColumnCount?: boolean;
	/**
	 * Discard inconsistent columns count when the record contains less fields than expected, default to false.
	 */
	relax_column_count_less?: boolean;
	relaxColumnCountLess?: boolean;
	/**
	 * Discard inconsistent columns count when the record contains more fields than expected, default to false.
	 */
	relax_column_count_more?: boolean;
	relaxColumnCountMore?: boolean;
	/**
	 * Preserve quotes inside unquoted field.
	 */
	relax_quotes?: boolean;
	relaxQuotes?: boolean;
	/**
	 * One or multiple characters used to delimit record rows; defaults to auto discovery if not provided.
	 * Supported auto discovery method are Linux ("\n"), Apple ("\r") and Windows ("\r\n") row delimiters.
	 */
	record_delimiter?: string | string[] | Buffer | Buffer[];
	recordDelimiter?: string | string[] | Buffer | Buffer[];
	/**
	 * If true, ignore whitespace immediately preceding the delimiter (i.e. right-trim all fields), defaults to false.
	 * Does not remove whitespace in a quoted field.
	 */
	rtrim?: boolean;
	/**
	 * Dont generate empty values for empty lines.
	 * Defaults to false
	 */
	skip_empty_lines?: boolean;
	skipEmptyLines?: boolean;
	/**
	 * Skip a line with error found inside and directly go process the next line.
	 */
	skip_records_with_error?: boolean;
	skipRecordsWithError?: boolean;
	/**
	 * Don't generate records for lines containing empty column values (column matching /\s*\/), defaults to false.
	 */
	skip_records_with_empty_values?: boolean;
	skipRecordsWithEmptyValues?: boolean;
	/**
	 * Stop handling records after the requested number of records.
	 */
	to?: number;
	/**
	 * Stop handling records after the requested line number.
	 */
	to_line?: number;
	toLine?: number;
	/**
	 * If true, ignore whitespace immediately around the delimiter, defaults to false.
	 * Does not remove whitespace in a quoted field.
	 */
	trim?: boolean;
}
declare interface Path {
	distance: number;
	predecessor: string;
}
declare type CastingDateFunction = (value: string, context: CastingContext) => Date;
declare type CastingFunction = (value: string, context: CastingContext) => any;
declare type ColumnOption = string | undefined | null | false | {
	name: string;
};
declare type CsvErrorCode = "CSV_INVALID_OPTION_BOM" | "CSV_INVALID_OPTION_CAST" | "CSV_INVALID_OPTION_CAST_DATE" | "CSV_INVALID_OPTION_COLUMNS" | "CSV_INVALID_OPTION_GROUP_COLUMNS_BY_NAME" | "CSV_INVALID_OPTION_COMMENT" | "CSV_INVALID_OPTION_DELIMITER" | "CSV_INVALID_OPTION_ON_RECORD" | "CSV_INVALID_CLOSING_QUOTE" | "INVALID_OPENING_QUOTE" | "CSV_INVALID_COLUMN_MAPPING" | "CSV_INVALID_ARGUMENT" | "CSV_INVALID_COLUMN_DEFINITION" | "CSV_MAX_RECORD_SIZE" | "CSV_NON_TRIMABLE_CHAR_AFTER_CLOSING_QUOTE" | "CSV_QUOTE_NOT_CLOSED" | "CSV_RECORD_INCONSISTENT_FIELDS_LENGTH" | "CSV_RECORD_INCONSISTENT_COLUMNS" | "CSV_OPTION_COLUMNS_MISSING_NAME";

declare {};
