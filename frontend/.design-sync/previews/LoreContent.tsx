import { LoreContent } from "frontend";

// The Lore tab: the operator's handbook archive, one collapsible per file, the
// first one open. The renderer parses each paragraph - `[Header]` lines become
// the two-column stat rows, quoted lines become blockquotes - so the fixture
// keeps the raw `handbook.storyTextAudio` strings from
// `/api/operators/char_4064_mlynar`.
const MLYNAR_HANDBOOK = [
    {
        "storyTitle": "Basic Info",
        "stories": [
            {
                "storyText": "[Code Name] Młynar\n[Gender] Male\n[Combat Experience] 17 Years\n[Place of Birth] Kazimierz\n[Date of Birth] Dec 3\n[Race] Kuranta\n[Height] 191cm\n[Infection Status]\nMedical tests have confirmed that no infection is present."
            }
        ]
    },
    {
        "storyTitle": "Physical Exam",
        "stories": [
            {
                "storyText": "[Physical Strength] Excellent\n[Mobility] Outstanding\n[Physical Resilience] Excellent\n[Tactical Acumen] Standard\n[Combat Skill] Outstanding\n[Originium Arts Assimilation] Excellent"
            }
        ]
    },
    {
        "storyTitle": "Profile",
        "stories": [
            {
                "storyText": "Młynar Nearl, the former head of the Nearl household and the uncle of Operators Nearl and Blemishine, has yet to receive a knight title of any kind.\nRoughly two months after the Kazimierz Major came to a close, he accepted Rhodes Island's invitation and is now assisting us with our affairs in Kazimierz."
            }
        ]
    },
    {
        "storyTitle": "Clinical Analysis",
        "stories": [
            {
                "storyText": "Imaging tests reveal clear, normal outlines of internal organs, and no abnormal shadows have been detected. Originium granules have not been detected in the circulatory system and there is no sign of infection. At this time, this operator is believed to be uninfected.\n\n[Cell-Originium Assimilation] 0%\nOperator Młynar shows no signs of Originium infection.\n\n[Blood Originium-Crystal Density] 0.14u/L\nAt one point, Operator Młynar spent an extended period of time in the wilderness. As such, his numbers are slightly higher than average for an office worker in the Grand Knight Territory.\nAlthough it is unlikely that he needs a reminder, the Medical Department has nonetheless provided him with recommendations on wilderness protection, per protocol."
            }
        ]
    },
    {
        "storyTitle": "Archive File 1",
        "stories": [
            {
                "storyText": "We rarely get the opportunity to see Operator Młynar aboard the landship, but if you are lucky enough to run into him and he happens to have some free time, he might just give you a few words of advice at the training grounds.\nMłynar may not have formally been a campaign knight, but his combat techniques clearly take after the traditional style employed by Kazimierz's knights. It can be inferred that the Nearl family's traditional training practices were sufficient to develop his mastery over the weapons commonly used by knights. As a young man, Młynar frequently followed his father, Kirill Nearl, and his team of campaign knights as an apprentice, giving him the opportunity to accumulate combat experience under the wings of a great many outstanding knights.\nHowever, whenever he puts down his training knight shield and picks up his sword, it becomes immediately clear to everyone nearby that the sword <i>is</i> his weapon. In most historied knight families, when a young man comes of age, he often chooses to once again leave home to improve his techniques and disposition, before joining a knight order to serve Kazimierz. However, having already mastered the sword during his time as a wanderer, Młynar set himself on a different path.\n...As for his Originium Arts, from what I've heard, knights never use Arts during a duel unless their beliefs are on the line. That said, during his test, he did show us his Arts just once.\nWe've seen Nearl and Blemishine's Arts. They are always as radiant as can be, possibly scorching, unadulterated. To our surprise, though, Operator Młynar's Arts initially felt gentle, even loving. However, upon realizing the anger the flow of his golden light embodied, we had to immediately stop him from drawing his sword, so as to minimize the repair work needed after his test."
            }
        ]
    },
    {
        "storyTitle": "Archive File 2",
        "stories": [
            {
                "storyText": "Unlike the Radiant Knight, Młynar primarily operates outside the Grand Knight Territory. Not long after he came aboard Rhodes Island and signed his agreement, he made his way back to Kazimierz. However, he did not immediately report to Rhodes Island's Kawalerielki office. In addition, he tendered his letter of resignation to his former employer in the Grand Knight Territory by mail.\nClearly, there are many in the city who view Młynar as a threat, and we can also surmise from his own account that a number of recent incidents he was involved in have affected the interests of certain individuals. Still, the retrial of a misjudged case, or the downfall of a few companies and nobles, are hardly significant in this city. At this point, rather than being concerned by all the attacks, open or covert, against his family, perhaps it's more fitting to say that having stayed in this city for ten years, he no longer feels that fighting these forces will accomplish anything.\n'We were honored to have his help during an operation. He wasn't as difficult to work with as I expected. By which I mean I thought he'd be a bossy, arrogant noble, or maybe someone who strictly followed the traditional rules of chivalry... but he simply listened and promised to give you a hand before you screwed up too badly. The only thing we had to get used to is that he never calls anyone by their codename. I mean, we do have a good number of Kazimierzian operators who use their titles from the Major as their codenames, so maybe he's got a grudge against that, but it's not that weird if you ask me.\n'...That said, it's hard not to feel bad if you make a mistake while working with him. That time, I almost screwed up the entire mission, and after that we let him make all the decisions.\n'He gives smooth, efficient plans. It doesn't seem like he cares whether you can actually follow them or not, though. I remember him asking us to split up to stop a couple commercial knights from tailing us. How were we supposed to find them? How would we stop them? Can we knock them out? Is it okay if we blow the roads up? He didn't discuss any of that with us.\n'I guess he's probably more used to working alone? I mean, all of us did our own thing and finished our own objectives with no communication whatsoever, and somehow it all just worked out... I'm really curious. What kind of man does it take to get used to doing things the way he does? And what got him doing things like that in the first place?'"
            }
        ]
    },
    {
        "storyTitle": "Archive File 3",
        "stories": [
            {
                "storyText": "[Classified Log]\nBased on available data, the Nearl couple was active on the battlefield for an exceedingly short period of time. The two of them rose to prominence twenty-five years ago when they fought under the same knight order's banner during the campaign knights' invasion of Ursus, with Schnitz taking over command during a crucial battle when the knight captain was gravely injured, and leaving a deep impression on the soldiers from both sides. However, that was their last notable accomplishment. Their disappearance without a trace over a decade ago seems to have marked a turning point in the fate of this generation of the Nearl family. As Rhodes Island has expressed our willingness to help locate them, Młynar agreed to have a conversation with us on the matter.\nDuring our conversation, we learned that Yolanta Nearl was also born into an established knight family and had been on good terms with the Nearls since she was young. She had originally intended to join the Silverlance Pegasus when she became a campaign knight, but Schnitz insisted that she would be more effective on the battlefield with a bow than a silverlance.\n'He arrived at that conclusion the moment she released her bowstring for the first time,' said Młynar, without elaborating further. Yolanta quickly proved Schnitz correct with her performance during the campaign, and her extraordinary talent in and understanding of Originium Arts made her particularly effective when deployed in positions overlooking the entire battlefield.\nAs for his brother Schnitz, Młynar had only this to say: 'He is my brother.'\nPerhaps it is because of how well he knew them, but despite the passage of time and the fact that everyone, including their daughters Margaret and Maria, has already accepted that they might be dead, Młynar continues to search for his brother and sister-in-law. When asked if he had any new leads, he shook his head, though it seemed his doubts on the matter were stronger than ever."
            }
        ]
    },
    {
        "storyTitle": "Archive File 4",
        "stories": [
            {
                "storyText": "Before joining Rhodes Island, Młynar worked at a medium-sized corporation that primarily sold construction materials. Mieszko Industries is its biggest shareholder, and the company has had a stable fiscal performance. It has not faced any significant charges and has no obvious connections to the nobility. It's merely one of Kazimierz's countless unremarkable companies. While prevailing rumors at the time suggested that the Nearls' younger son would join either the Adeptus Sprawiedliwi or the Knights Association and eventually take over the family's Knight Primus title from the ailing Kirill, the decision that Młynar ultimately made was perhaps influenced by some external factor, or his disappointment with Kazimierz.\nBut we've met many disappointed men. The real question is what made him decide to do something <i>now</i>, and how much of his strength he's willing to lend to Rhodes Island.\nMłynar joined Rhodes Island after the Infected disturbances across the Dzwonek region. The Adeptus Sprawiedliwi successfully kept the word of the incident from spreading, thus ensuring no other forces could take advantage of this information. Młynar himself did not provide much in the way of his personal perspective on the matter. However, when HR brought this up again while organizing files, he gave an unexpected response: 'I've just always thought I needed to prove Kazimierz still has a way out.'\nMłynar has also never brought up anything to do with Operators Blemishine, Whislash, or Platinum since signing his cooperation agreement with us. Nor does he interact with Flametail and the others while in Kazimierz. There has been only one instance of Młynar, Whislash, and Blemishine having a conversation, after Whislash requested a meeting, and it lasted less than ten minutes. Although the details of that conversation are unknown, our Engineering operators told us that Blemishine suddenly seemed much more energetic, as if a weight had been lifted from her heart. Even then, Młynar didn't appear to pay much more attention to the other Kazimierzian operators than before, even though many of us are just as curious about him as we are about the Radiant Knight herself.\nIt is clear that Młynar is not looking for an answer from Rhodes Island, and perhaps he does not even believe that there is an answer out there. But when he assisted us with a string of Infected problems, he displayed a sense of responsibility that definitely did not seem to come from respect towards his work. Some may believe the Land of Knights has no more room for knightly fantasies, and that the endless waves will eventually consume its cities, but how can anyone know how the next story will be written, and where the next city will be built, until they are completed?"
            }
        ]
    },
    {
        "storyTitle": "Promotion Record",
        "stories": [
            {
                "storyText": "[An old letter]\nI don't care, Father. Must knights prove themselves through victory? The men in the arenas never seem more brilliant simply because they manage to maul more of their opponents.\nKnights should stand with those in distress, those with nothing but the clothes on their backs. This alone I will always believe in. When you stand with the weak, failure is hardly unheard of. Rather than relishing in victory, I would rather believe that the spirit of the knighthood also encompasses the endurance of failure.\nIt's just as our family motto goes, 'Fear neither hardship nor darkness.'"
            }
        ]
    }
];

// Same operator, archive files first, so the open section is long-form prose
// rather than the dossier stat block.
const MLYNAR_ARCHIVE_FILES = [
    {
        "storyTitle": "Archive File 2",
        "stories": [
            {
                "storyText": "Unlike the Radiant Knight, Młynar primarily operates outside the Grand Knight Territory. Not long after he came aboard Rhodes Island and signed his agreement, he made his way back to Kazimierz. However, he did not immediately report to Rhodes Island's Kawalerielki office. In addition, he tendered his letter of resignation to his former employer in the Grand Knight Territory by mail.\nClearly, there are many in the city who view Młynar as a threat, and we can also surmise from his own account that a number of recent incidents he was involved in have affected the interests of certain individuals. Still, the retrial of a misjudged case, or the downfall of a few companies and nobles, are hardly significant in this city. At this point, rather than being concerned by all the attacks, open or covert, against his family, perhaps it's more fitting to say that having stayed in this city for ten years, he no longer feels that fighting these forces will accomplish anything.\n'We were honored to have his help during an operation. He wasn't as difficult to work with as I expected. By which I mean I thought he'd be a bossy, arrogant noble, or maybe someone who strictly followed the traditional rules of chivalry... but he simply listened and promised to give you a hand before you screwed up too badly. The only thing we had to get used to is that he never calls anyone by their codename. I mean, we do have a good number of Kazimierzian operators who use their titles from the Major as their codenames, so maybe he's got a grudge against that, but it's not that weird if you ask me.\n'...That said, it's hard not to feel bad if you make a mistake while working with him. That time, I almost screwed up the entire mission, and after that we let him make all the decisions.\n'He gives smooth, efficient plans. It doesn't seem like he cares whether you can actually follow them or not, though. I remember him asking us to split up to stop a couple commercial knights from tailing us. How were we supposed to find them? How would we stop them? Can we knock them out? Is it okay if we blow the roads up? He didn't discuss any of that with us.\n'I guess he's probably more used to working alone? I mean, all of us did our own thing and finished our own objectives with no communication whatsoever, and somehow it all just worked out... I'm really curious. What kind of man does it take to get used to doing things the way he does? And what got him doing things like that in the first place?'"
            }
        ]
    },
    {
        "storyTitle": "Archive File 3",
        "stories": [
            {
                "storyText": "[Classified Log]\nBased on available data, the Nearl couple was active on the battlefield for an exceedingly short period of time. The two of them rose to prominence twenty-five years ago when they fought under the same knight order's banner during the campaign knights' invasion of Ursus, with Schnitz taking over command during a crucial battle when the knight captain was gravely injured, and leaving a deep impression on the soldiers from both sides. However, that was their last notable accomplishment. Their disappearance without a trace over a decade ago seems to have marked a turning point in the fate of this generation of the Nearl family. As Rhodes Island has expressed our willingness to help locate them, Młynar agreed to have a conversation with us on the matter.\nDuring our conversation, we learned that Yolanta Nearl was also born into an established knight family and had been on good terms with the Nearls since she was young. She had originally intended to join the Silverlance Pegasus when she became a campaign knight, but Schnitz insisted that she would be more effective on the battlefield with a bow than a silverlance.\n'He arrived at that conclusion the moment she released her bowstring for the first time,' said Młynar, without elaborating further. Yolanta quickly proved Schnitz correct with her performance during the campaign, and her extraordinary talent in and understanding of Originium Arts made her particularly effective when deployed in positions overlooking the entire battlefield.\nAs for his brother Schnitz, Młynar had only this to say: 'He is my brother.'\nPerhaps it is because of how well he knew them, but despite the passage of time and the fact that everyone, including their daughters Margaret and Maria, has already accepted that they might be dead, Młynar continues to search for his brother and sister-in-law. When asked if he had any new leads, he shook his head, though it seemed his doubts on the matter were stronger than ever."
            }
        ]
    },
    {
        "storyTitle": "Promotion Record",
        "stories": [
            {
                "storyText": "[An old letter]\nI don't care, Father. Must knights prove themselves through victory? The men in the arenas never seem more brilliant simply because they manage to maul more of their opponents.\nKnights should stand with those in distress, those with nothing but the clothes on their backs. This alone I will always believe in. When you stand with the weak, failure is hardly unheard of. Rather than relishing in victory, I would rather believe that the spirit of the knighthood also encompasses the endurance of failure.\nIt's just as our family motto goes, 'Fear neither hardship nor darkness.'"
            }
        ]
    }
];

export const DossierFirst = () => <LoreContent operator={{ id: "char_4064_mlynar", name: "Młynar", handbook: { storyTextAudio: MLYNAR_HANDBOOK } }} />;

export const NarrativeFile = () => <LoreContent operator={{ id: "char_4064_mlynar", name: "Młynar", handbook: { storyTextAudio: MLYNAR_ARCHIVE_FILES } }} />;

export const NoRecords = () => <LoreContent operator={{ id: "char_4093_frston", name: "Friston-3", handbook: { storyTextAudio: [] } }} />;
