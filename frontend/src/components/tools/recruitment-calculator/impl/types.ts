export interface RecruitmentTag {
  id: string
  name: string
  type: string
}

export interface OperatorOutcome {
  name: string
  rarity: number
  guaranteed: boolean
  tags: string[]
}

export type TagGroup = Record<string, RecruitmentTag[]>
