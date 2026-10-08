import { Calcs } from "frontend";

// The Originite Prime ledger under a planner row: income, expenses, and the
// balance (income - expenses).

// In credit: 66 earned (event first clears and the monthly card), 36 spent on a
// skin, 30 left over.
export const InCredit = () => (
    <div className="w-56 p-4">
        <Calcs total={{ income: 66, expense: 36, balance: 30 }} />
    </div>
);

// Overspent: two 18 OP skins and an 18 OP outfit against 24 income goes negative,
// and the balance turns destructive.
export const Overspent = () => (
    <div className="w-56 p-4">
        <Calcs total={{ income: 24, expense: 54, balance: -30 }} />
    </div>
);

// No plan yet: every line reads zero.
export const Empty = () => (
    <div className="w-56 p-4">
        <Calcs total={undefined} />
    </div>
);
