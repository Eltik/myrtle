export interface OperatorNote {
    id: string;
    operator_id: string;
    pros: string;
    cons: string;
    notes: string;
    trivia: string;
    summary: string;
    tags: string[];
    operator_name?: string;
    operator_rarity?: string;
    operator_profession?: string;
    created_at: string;
    updated_at: string;
}
