import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

/**
 * Every string the Pulls tab renders. The sub-panels import this module type-only
 * and intersect it into their own `TypedT`, the same way `plan.messages.ts` is
 * shared across the Originite Prime planner's components.
 *
 * Arknights vocabulary stays untranslated on purpose: Orundum, Originite Prime,
 * Headhunting Permit, Annihilation and the operator names all come from game data.
 */
export const messages = {
    "release.pulls.modelNote": {
        text: "Rates come from the game's own banner data: a 2% base 6★ rate, +2% per roll after 50 without one, and the rate-up split declared for each banner type. Recurring income is modelled; one-off event Orundum is not, so add it under Extra per day.",
        description: "Explains where the numbers come from and what the model leaves out. '6★' means a six-star operator; 'rate-up' is the boosted-chance operator on a banner.",
    },

    "release.pulls.resources.title": {
        text: "What you have",
        description: "Section heading above the starting-resource inputs.",
    },
    "release.pulls.resources.orundum": {
        text: "Orundum",
        description: "Input label. Orundum is the game's headhunting currency; do not translate.",
    },
    "release.pulls.resources.permits": {
        text: "Permits",
        description: "Input label for single Headhunting Permits, each worth one pull.",
    },
    "release.pulls.resources.tenPermits": {
        text: "Ten-roll permits",
        description: "Input label for Ten-roll Headhunting Permits, each worth ten pulls.",
    },
    "release.pulls.resources.originite": {
        text: "Originite Prime",
        description: "Input label. Originite Prime is the premium currency; do not translate.",
    },
    "release.pulls.resources.pity": {
        text: "Pulls since last 6★",
        description: "Input label for the pity counter: how many draws since the last six-star operator.",
    },
    "release.pulls.resources.pityHint": {
        text: "Counts on Standard and Kernel banners, which share a counter. Limited and collab banners always start you at zero.",
        description: "Hint under the pity input. 'Standard', 'Kernel', 'Limited' and collab are Arknights banner families.",
    },
    "release.pulls.resources.sync": {
        text: "Use my account",
        description: "Button that fills the resource inputs from the signed-in player's synced account.",
    },
    "release.pulls.resources.synced": {
        text: "Filled from your last sync.",
        description: "Note shown when the resource inputs came from the account rather than being typed.",
    },
    "release.pulls.resources.signedOut": {
        text: "Sign in to fill these from your account.",
        description: "Note shown to signed-out visitors next to the resource inputs.",
    },

    "release.pulls.income.title": {
        text: "What you earn",
        description: "Section heading above the recurring-income settings.",
    },
    "release.pulls.income.monthlyCard": {
        text: "Monthly card",
        description: "Switch label for the paid monthly subscription, which pays 200 Orundum a day.",
    },
    "release.pulls.income.spendOriginite": {
        text: "Convert Originite Prime",
        description: "Switch label: whether to count Originite Prime as pulls at 180 Orundum each.",
    },
    "release.pulls.income.store": {
        text: "Monthly store",
        description: "Switch label for the monthly certificate-shop Orundum and permits.",
    },
    "release.pulls.income.annihilation": {
        text: "Annihilation per week",
        description: "Label for the weekly Annihilation Orundum cap, which rises with campaign progress.",
    },
    "release.pulls.income.extra": {
        text: "Extra per day",
        description: "Input label for Orundum per day the model does not itself cover.",
    },
    "release.pulls.income.goldCerts": {
        text: "Gold certs per day",
        description: "Input label for Distinction Certificates earned per day. 'Distinction Certificate' is the in-game name; players call them gold certs.",
    },
    "release.pulls.income.goldCertShop": {
        text: "Buy permits with gold certs",
        description: "Switch label: spend Distinction Certificates on the monthly Headhunting Permit bundles.",
    },
    "release.pulls.income.goldCertHint": {
        text: "The monthly ladder is 38 permits for 258 certs, bought cheapest rung first. There is no official earn rate, so this is your own estimate; 1.5 a day suits a year-old account.",
        description: "Hint under the gold certificate inputs, explaining the shop ladder and why the rate is user-supplied.",
    },
    "release.pulls.income.greenCerts": {
        text: "Green certs per week",
        description: "Input label for Commendation Certificates earned per week. 'Commendation Certificate' is the in-game name; players call them green certs.",
    },
    "release.pulls.income.greenCertShop": {
        text: "Green cert shop",
        description: "Label for how deep into the monthly Commendation Certificate shop the player buys.",
    },
    "release.pulls.income.greenShop.off": {
        text: "Not buying",
        description: "Option: do not spend Commendation Certificates on pulls at all.",
    },
    "release.pulls.income.greenShop.phase1": {
        text: "Phase 1 only",
        description: "Option: buy only from the first phase of the monthly Commendation Certificate shop.",
    },
    "release.pulls.income.greenShop.phase2": {
        text: "Phase 1 and 2",
        description: "Option: buy through both phases of the monthly Commendation Certificate shop.",
    },
    "release.pulls.income.greenCertHint": {
        text: "Phase 1 sells 2 permits at 240 certs each and 600 Orundum at 240 certs, which are worth exactly the same. Phase 2 adds 2 permits at 450 each, but only opens after phase 1 is bought out entirely at 1490 certs, so weekly missions alone will not reach it.",
        description: "Hint under the green certificate inputs, explaining the phase gate and the equal value of the two phase 1 rows.",
    },
    "release.pulls.income.freePulls": {
        text: "Count banner free pulls",
        description: "Switch label: include the free pulls a Limited or collab banner hands out.",
    },
    "release.pulls.income.freePullsHint": {
        text: "A Limited banner gives a free ten-roll plus one free pull a day, which is 24 over a 14-day run. A collab gives two ten-rolls. These expire with the banner, so they are counted on it rather than banked.",
        description: "Hint explaining where banner free pulls come from and why they are not savings.",
    },
    "release.pulls.income.skinOriginiteHint": {
        text: "{count, plural, =0 {No outfits picked in the Planner tab.} one {# outfit picked in the Planner tab costs {op} Originite Prime.} other {# outfits picked in the Planner tab cost {op} Originite Prime.}}",
        description: "Hint showing what the skins planner has committed. 'Planner tab' names the sibling tab, so keep it matching.",
    },
    "release.pulls.income.originiteContested": {
        text: "Converting Originite Prime spends all of it on pulls, leaving nothing for the {needed} your outfit picks need.",
        description: "Warning that the pull plan and the outfit plan are claiming the same Originite Prime.",
    },
    "release.pulls.income.originiteShort": {
        text: "Your outfit picks need {needed} Originite Prime and you will only have {available}.",
        description: "Warning that the outfit picks alone cost more Originite Prime than the player will hold.",
    },
    "release.pulls.income.skinUnpriced": {
        text: "{count, plural, one {# pick has no price yet, so this is an underestimate.} other {# picks have no price yet, so this is an underestimate.}}",
        description: "Caveat when some picked outfits have no known price.",
    },
    "release.pulls.summary.free": {
        text: "Free on banners",
        description: "Stat label for pulls the banners give away, which cannot be saved.",
    },
    "release.pulls.income.extraHint": {
        text: "Event rewards, mail and anything else. This project's release data carries Originite Prime for event stages but no event Orundum, so it is not guessed at here.",
        description: "Hint under the Extra per day input, explaining why event Orundum must be entered by hand.",
    },
    "release.pulls.income.weekly": {
        text: "{count, plural, one {# pull} other {# pulls}} per week",
        description: "Summary of recurring income expressed as pulls per week.",
    },
    "release.pulls.income.horizon": {
        text: "Project ahead",
        description: "Label for how far into the future the projection runs.",
    },
    "release.pulls.income.horizonDays": {
        text: "{count, plural, one {# day} other {# days}}",
        description: "Option in the projection-horizon picker.",
    },

    "release.pulls.summary.now": {
        text: "Pulls now",
        description: "Stat label: pulls affordable with current resources.",
    },
    "release.pulls.summary.horizon": {
        text: "By {date}",
        description: "Stat label: pulls affordable by the end of the projection, with the date.",
    },
    "release.pulls.summary.perWeek": {
        text: "Earned per week",
        description: "Stat label for recurring weekly Orundum income.",
    },
    "release.pulls.summary.withOriginite": {
        text: "{count} with Originite Prime",
        description: "Sub-label showing the higher pull count once Originite Prime is converted.",
    },

    "release.pulls.chart.title": {
        text: "Pulls over time",
        description: "Heading for the line chart of projected pulls.",
    },
    "release.pulls.chart.pulls": {
        text: "Pulls",
        description: "Chart series name for pulls excluding Originite Prime.",
    },
    "release.pulls.chart.withOriginite": {
        text: "With Originite Prime",
        description: "Chart series name for pulls including converted Originite Prime.",
    },
    "release.pulls.chart.net": {
        text: "After your plan",
        description: "Chart series name for the balance left once the plan's commitments are taken out.",
    },
    "release.pulls.chart.aria": {
        text: "Projected pulls from today through {date}.",
        description: "Screen-reader description of the projection chart.",
    },
    "release.pulls.chart.ariaPlanned": {
        text: "Projected pulls from today through {date}, and the balance left after the {count, plural, one {# banner} other {# banners}} your plan commits to.",
        description: "Screen-reader description of the projection chart once the plan spends on at least one banner.",
    },

    "release.pulls.plan.title": {
        text: "Your plan",
        description: "Heading for the table where the user commits pulls to upcoming banners.",
    },
    "release.pulls.plan.intro": {
        text: "Commit pulls to the banners you want. Everything you spend comes out of the same pool, so a banner you save for is one you can still afford later.",
        description: "Explains that allocating pulls to one banner reduces what is left for the others.",
    },
    "release.pulls.plan.allocate": {
        text: "Pulls to spend",
        description: "Input label for how many pulls to commit to a banner.",
    },
    "release.pulls.plan.max": {
        text: "Spend everything",
        description: "Accessible name for the control committing every remaining pull to this banner. The visible control shows the number alone.",
    },
    "release.pulls.plan.potMore": {
        text: "One more copy of {operator}",
        description: "Accessible name for the control raising how many copies of an operator to pull for.",
    },
    "release.pulls.plan.potLess": {
        text: "One fewer copy of {operator}",
        description: "Accessible name for the control lowering how many copies of an operator to pull for.",
    },
    "release.pulls.plan.potValue": {
        text: "P{count}",
        description: "An operator's wanted potential, shown on the stepper. P6 is maximum potential. 'P' abbreviates Potential, the in-game term for duplicate upgrades.",
    },
    "release.pulls.plan.potNone": {
        text: "None",
        description: "Shown on the potential stepper when the user is not pulling for that operator.",
    },
    "release.pulls.plan.yourGoal": {
        text: "Your goal",
        description: "Row label for the estimate of exactly what the user asked for on this banner.",
    },
    "release.pulls.plan.pickHint": {
        text: "Set the potential you want on each operator, and the pulls follow. P6 is maximum potential.",
        description: "Hint shown once above the banner list. 'Potential' is the in-game term for duplicate upgrades, P1 through P6.",
    },
    "release.pulls.plan.inferredCount": {
        text: "{count, plural, one {# banner has} other {# banners have}} no published rate-up detail yet, so the split is inferred from the banner type.",
        description: "Caveat shown once above the list, counting the banners whose rate-up share was inferred.",
    },
    "release.pulls.plan.freeOnBanner": {
        text: "+{count} free",
        description: "Compact note that a banner hands out this many free pulls of its own.",
    },
    "release.pulls.plan.setTo": {
        text: "Set to",
        description: "Label before the buttons that fill in a preset number of pulls.",
    },
    "release.pulls.plan.availableHere": {
        text: "available",
        description: "Unit beside the headline figure on a banner row: the pulls the player has to spend on it.",
    },
    "release.pulls.plan.plannedHere": {
        text: "planned",
        description: "Unit beside the headline figure on a banner row once a pull count has been committed: it echoes the number in the input below.",
    },
    "release.pulls.plan.planFits": {
        text: "{available} available",
        description: "Clause beside a banner row's headline when the bank covers what was committed.",
    },
    "release.pulls.plan.planShort": {
        text: "{short} short, you will only have {available}",
        description: "Clause beside a banner row's headline when the commitment overran the bank.",
    },
    "release.pulls.plan.clear": {
        text: "Clear",
        description: "Button removing this banner's commitment.",
    },
    "release.pulls.plan.spark": {
        text: "Spark {count}",
        description: "Button committing exactly the pulls needed for a Limited banner's outright exchange.",
    },
    "release.pulls.plan.guarantee": {
        text: "Guarantee {count}",
        description: "Button committing exactly the pulls at which a forced rate-up binds.",
    },
    "release.pulls.plan.estimated": {
        text: "Estimated",
        description: "Column heading for how many pulls this banner is expected to need for its rate-up operator.",
    },
    "release.pulls.plan.estimatedDetailOpen": {
        text: "{mean}+ on average, more than {horizon} if unlucky",
        description: "Secondary detail under the estimate when nine runs in ten do not finish inside the horizon, so there is no 90th-percentile figure to give and the mean is a floor.",
    },
    "release.pulls.plan.estimatedDetail": {
        text: "{mean} on average, {p90} if unlucky",
        description: "Secondary detail under the estimate: the mean and the 90th-percentile pull count.",
    },
    "release.pulls.plan.goal.specific": {
        text: "This operator",
        description: "Row label in the estimate: reaching one named rate-up operator.",
    },
    "release.pulls.plan.goal.any": {
        text: "Either one",
        description: "Row label in the estimate: reaching whichever of the two rate-up operators comes first.",
    },
    "release.pulls.plan.goal.both": {
        text: "Both",
        description: "Row label in the estimate: reaching both rate-up operators.",
    },
    "release.pulls.plan.goal.maxPot": {
        text: "Max pot",
        description: "Row label in the estimate: taking the operator to maximum potential, which is six copies.",
    },
    "release.pulls.plan.goalPick": {
        text: "Set pulls to {count} for {goal}",
        description: "Accessible name for an estimate row, which fills in that many pulls when clicked.",
    },
    "release.pulls.plan.goalValue": {
        text: "~{count}",
        description: "A pull count in the estimate rows. The tilde marks it as an estimate.",
    },
    "release.pulls.plan.estimatedTail": {
        text: "{percent} of runs need more than {horizon}",
        description: "Caveat that some simulated runs do not get the operator within the estimate's horizon.",
    },
    "release.pulls.plan.sumCovers": {
        text: "{total} covers ~{goal}",
        description: "Verdict under the estimates: what will be thrown at this banner reaches the highlighted goal.",
    },
    "release.pulls.plan.sumGap": {
        text: "{total}, {gap} under ~{goal}",
        description: "Verdict under the estimates: what will be thrown at this banner falls short of the highlighted goal.",
    },
    "release.pulls.plan.sumFreeCovers": {
        text: "{spent} + {free} free = {total}, covers ~{goal}",
        description: "Verdict under the estimates when the banner hands out free pulls, showing the sum rather than counting them silently.",
    },
    "release.pulls.plan.sumFreeGap": {
        text: "{spent} + {free} free = {total}, {gap} under ~{goal}",
        description: "Verdict under the estimates when the banner hands out free pulls and the total still falls short.",
    },
    "release.pulls.plan.reset": {
        text: "Clear plan",
        description: "Button removing every banner commitment at once.",
    },
    "release.pulls.plan.export": {
        text: "Export CSV",
        description: "Button downloading the plan as a spreadsheet file.",
    },
    "release.pulls.plan.committed": {
        text: "Committed",
        description: "Stat label for the total pulls the plan spends.",
    },
    "release.pulls.plan.remaining": {
        text: "Left over",
        description: "Stat label for pulls unspent at the end of the plan.",
    },
    "release.pulls.plan.overrun": {
        text: "{count, plural, one {# banner} other {# banners}} over budget",
        description: "Warning summarising how many banners the plan cannot pay for in full.",
    },
    "release.pulls.plan.untouched": {
        text: "Nothing committed yet. Use Max or Spark on a banner to start planning.",
        description: "Hint shown when the plan is empty.",
    },
    "release.pulls.banners.empty": {
        text: "No upcoming banner has a resolved EN date yet.",
        description: "Empty state for the banner table. 'EN' is the English game server.",
    },
    "release.pulls.banners.sparkMet": {
        text: "Clears the {spark}-pull guarantee",
        description: "Note that the projected budget reaches a Limited banner's outright exchange.",
    },
    "release.pulls.banners.inferred": {
        text: "Rate-up split inferred from the banner type; this pool has no published detail yet.",
        description: "Caveat shown when a banner's exact rate-up share was not available and was derived from its type.",
    },

    "release.pulls.odds.title": {
        text: "Odds calculator",
        description: "Heading for the panel that computes exact pull probabilities.",
    },
    "release.pulls.odds.banner": {
        text: "Banner type",
        description: "Picker label for which kind of banner to compute odds against.",
    },
    "release.pulls.odds.pulls": {
        text: "Pulls",
        description: "Input label for how many draws to compute the odds over.",
    },
    "release.pulls.odds.fromBudget": {
        text: "Use my projection",
        description: "Button that sets the pull count from the budget projection.",
    },
    "release.pulls.odds.atPulls": {
        text: "{count, plural, one {# pull} other {# pulls}}",
        description: "Tooltip heading on the odds curve, naming the number of draws at that point.",
    },
    "release.pulls.odds.specific": {
        text: "A specific rate-up",
        description: "Result label: chance of at least one copy of one named featured operator.",
    },
    "release.pulls.odds.any": {
        text: "Any rate-up",
        description: "Result label: chance of at least one featured operator of any kind.",
    },
    "release.pulls.odds.both": {
        text: "Both rate-ups",
        description: "Result label: chance of getting both featured operators.",
    },
    "release.pulls.odds.expectedSix": {
        text: "Expected 6★",
        description: "Result label: the average number of six-star operators these pulls yield.",
    },
    "release.pulls.odds.copies": {
        text: "Copies of one rate-up",
        description: "Heading for the distribution of how many duplicates you end up with.",
    },
    "release.pulls.odds.copiesRow": {
        text: "{count, plural, =0 {None} one {# copy} other {# copies}}",
        description: "Row label in the copy-count distribution.",
    },
    "release.pulls.odds.method": {
        text: "Exact, not simulated: the full distribution over pity states is carried forward one roll at a time.",
        description: "Note explaining that the odds are computed exactly rather than sampled. 'Pity' is the rising-chance mechanic.",
    },

    "release.pulls.sim.title": {
        text: "Pull simulator",
        description: "Heading for the panel that rolls the banner for real.",
    },
    "release.pulls.sim.desc": {
        text: "Roll against the same model, with pity and guarantees carrying between clicks exactly as they do in game.",
        description: "Subheading for the simulator panel.",
    },
    "release.pulls.sim.target": {
        text: "Copies wanted",
        description: "Input label for how many copies of the featured operator to pull for.",
    },
    "release.pulls.sim.pullOne": {
        text: "Pull",
        description: "Button that draws a single pull in the simulator.",
    },
    "release.pulls.sim.pullTen": {
        text: "Pull 10",
        description: "Button that draws ten pulls in the simulator.",
    },
    "release.pulls.sim.reset": {
        text: "Reset",
        description: "Button that clears the simulator back to a fresh account state.",
    },
    "release.pulls.sim.spent": {
        text: "Pulls spent",
        description: "Stat label for how many draws the current simulator run has used.",
    },
    "release.pulls.sim.pityNow": {
        text: "Since last 6★",
        description: "Stat label for the simulator's current pity counter.",
    },
    "release.pulls.sim.got": {
        text: "Rate-up copies",
        description: "Stat label for how many featured operators the run has produced.",
    },
    "release.pulls.sim.cost": {
        text: "{count} Orundum",
        description: "What the current simulator run would have cost.",
    },
    "release.pulls.sim.guaranteed": {
        text: "Guaranteed",
        description: "Badge on a simulated pull that a banner guarantee decided rather than chance.",
    },
    "release.pulls.sim.distribution": {
        text: "Across {runs} runs",
        description: "Heading for the sampled distribution of pulls needed.",
    },
    "release.pulls.sim.median": {
        text: "Median",
        description: "Stat label for the 50th percentile of pulls needed.",
    },
    "release.pulls.sim.p90": {
        text: "Unlucky (90th)",
        description: "Stat label for the 90th percentile of pulls needed, i.e. a bad run.",
    },
    "release.pulls.sim.withinBudget": {
        text: "Within your {count, plural, one {# pull} other {# pulls}}",
        description: "Stat label for the share of runs that succeed inside the projected budget.",
    },

    "release.pulls.banner.LIMITED": {
        text: "Limited",
        description: "Banner type: the paid Limited/Celebration banner featuring a new limited operator.",
    },
    "release.pulls.banner.SINGLE": {
        text: "Standard, one rate-up",
        description: "Banner type: rotating Standard Headhunting with a single featured 6-star.",
    },
    "release.pulls.banner.DOUBLE": {
        text: "Standard, two rate-ups",
        description: "Banner type: rotating Standard Headhunting with two featured 6-stars.",
    },
    "release.pulls.banner.NORMAL": {
        text: "Standard",
        description: "Banner type: the general Standard Headhunting pool.",
    },
    "release.pulls.banner.LINKAGE": {
        text: "Joint Operation, collab",
        description: "Banner type: a crossover/collaboration banner. 'Joint Operation' is the in-game name.",
    },
    "release.pulls.banner.CLASSIC": {
        text: "Kernel Locating",
        description: "Banner type: the in-game name for the older-operator rerun banner.",
    },
    "release.pulls.banner.ATTAIN": {
        text: "Special Headhunting",
        description: "Banner type: the in-game name for the banner whose first 6-star is an operator you do not own.",
    },
    "release.pulls.odds.copiesTail": {
        text: "{count, plural, other {# or more}}",
        description: "Label for the last row of the copy-count chart, which collects every higher count.",
    },
    "release.pulls.a11y.horizon": {
        text: "Projection horizon",
        description: "Accessible name for the control choosing how far ahead to project.",
    },
    "release.pulls.a11y.chartOdds": {
        text: "Chance of the rate-up operator against the number of pulls, from 0 to {max} pulls.",
        description: "Screen-reader description of the odds curve.",
    },
    "release.pulls.a11y.chartSim": {
        text: "How many pulls each simulated run needed. Median {median}, 90th percentile {p90}.",
        description: "Screen-reader description of the simulated distribution chart.",
    },
    "release.pulls.sim.rateUpChip": {
        text: "Rate-up",
        description: "Label marking a simulated pull that produced a featured operator.",
    },
    "release.pulls.rule.share": {
        text: "{percent}% of 6★ pulls go to {count, plural, one {the rate-up} other {the # rate-ups}}",
        description: "Describes how much of the six-star rate a banner's featured operators take.",
    },
    "release.pulls.rule.carry": {
        text: "Shares a pity counter with other {scope} banners",
        description: "Note that pity carries across a banner family. 'scope' is Standard or Kernel.",
    },
    "release.pulls.rule.isolated": {
        text: "Pity starts at zero and is cleared when the banner ends",
        description: "Note that a banner tracks pity on its own.",
    },
    "release.pulls.rule.scope.standard": {
        text: "Standard",
        description: "Name of the rotating Standard Headhunting banner family.",
    },
    "release.pulls.rule.scope.kernel": {
        text: "Kernel",
        description: "Name of the Kernel Locating banner family for older operators.",
    },
    "release.pulls.rule.guarantee.linkage": {
        text: "The collab operator is handed over on pull {at, number}",
        description: "Describes the collaboration banner's hard guarantee.",
    },
    "release.pulls.rule.guarantee.selection": {
        text: "Past {first, plural, one {# pull} other {# pulls}} the next 6★ is forced on-rate, and past {second} it is the other one",
        description: "Describes the Standard Selection guarantee on two-rate-up banners.",
    },
    "release.pulls.rule.guarantee.attain": {
        text: "The first 6★ is an operator you do not own",
        description: "Describes the Special Headhunting guarantee.",
    },
    "release.pulls.rule.spark": {
        text: "{count, plural, one {# pull exchanges} other {# pulls exchange}} for the featured operator outright",
        description: "Describes the Limited banner's certificate exchange.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
