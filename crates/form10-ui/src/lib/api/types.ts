export interface SettingsInput {
  financialYear: string;
  dcmpu: string;
  district: string;
  society: string;
  societyCode: string;
  oldMember: string;
  oldSociety: string;
  oldUnion: string;
  newMember: string;
  newSociety: string;
  newUnion: string;
  newFromMonth: number;
}

export interface SourceInfo {
  path: string;
  fileName: string;
  sheetName: string;
  financialYear: string | null;
  reportingPeriod: string | null;
  reportingMonths: string;
  reportingMonthIndices: number[];
  dcmpu: string | null;
  district: string | null;
  society: string | null;
  societyCode: string | null;
  memberCount: number;
}

export interface Summary {
  memberCount: number;
  activeSubscriptions: number;
  memberContribution: number;
  societyContribution: number;
  unionContribution: number;
  totalContribution: number;
}

export interface ExportPreview {
  destinationPath: string;
  destinationExists: boolean;
  suggestedFileName: string;
}
