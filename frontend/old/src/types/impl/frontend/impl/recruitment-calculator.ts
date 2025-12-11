import type { Operator } from "../../api/static/operator";

// Define expected data structures
export interface Tag {
    tagId: string;
    tagName: string;
    tagGroup: number;
    tagCat?: string;
}

// Updated OperatorOutcome structure
export type OperatorOutcome = {
    label: string[];
    operators: (Operator & { recruitOnly: boolean })[];
};

// Use Record for better type safety
export type GroupedTags = Record<string, Tag[]>;

// Define the expected structure of the API response (Success)
export interface ApiResponse<T> {
    data: T;
    error?: string; // Should ideally not be present on success
}

// Define a potential structure for error responses from the API proxy
export interface ErrorResponse {
    error?: string;
}

// Type for storing info about the clicked operator for tag display
export interface SelectedOpInfo {
    outcomeIndex: number;
    operatorId: string;
    allTags: string[]; // Store pre-calculated tags here
}
