import OperatorUnit from "../classes/operator-unit";
import fetchArtificers from "./impl/artificers";

export const ALL_OPERATORS: OperatorUnit[] = [];

export const init = async () => {
    await fetchArtificers();
};
