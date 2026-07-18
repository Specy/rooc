/** The stage at which a fluent model operation failed. */
export type ModelBuilderStage =
    | "declaration"
    | "construction"
    | "serialization"
    | "compilation"
    | "linearization"
    | "solving";

export type ModelBuilderErrorDetails = {
    source?: string;
    cause?: unknown;
    context?: readonly unknown[];
};

/** Error raised while declaring, constructing, compiling, or solving a fluent model. */
export class ModelBuilderError extends Error {
    readonly stage: ModelBuilderStage;
    readonly source?: string;
    readonly cause?: unknown;
    readonly context?: readonly unknown[];

    constructor(
        message: string,
        stage: ModelBuilderStage,
        details: ModelBuilderErrorDetails = {},
    ) {
        super(message);
        this.name = "ModelBuilderError";
        this.stage = stage;
        this.source = details.source;
        this.cause = details.cause;
        this.context = details.context;
    }
}
