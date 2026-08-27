#!/usr/bin/env python3
"""Refine the page-camera scale k, and cross-check the centre two ways.

WHY. A recovered centre solves as `ccx + (W/2+dx)/(k*sc) - W/(2*sc)`, whose sensitivity to k runs
892 to 1190 px per unit k on these subjects. A 0.02 search grid therefore injects 17.8 to 23.8 px
into every centre, which is larger than the 10 px envelope the anchors were being judged against.
Refining k to 0.002 with a parabola fit on the NCC peak drops that to 0.1 to 2.4 px.

🚨 THE RULE THIS ENCODES: never report a derived position without propagating the uncertainty its
search step implies. A "3.10 px spread" read off a 0.02 grid was four numbers quantised more
coarsely than the thing they were compared against, and it read as precision.

The second half solves each centre again with k PINNED to a candidate rule's prediction, which
removes k's error from the centre entirely. If free-k and pinned-k centres agree inside the
propagated error, that is evidence for the rule and the anchors at once; if they disagree, the rule
is doing work it has not earned. They disagreed: reed2 by 66.1 and 92.1 px against a 0.1 px error.
"""
