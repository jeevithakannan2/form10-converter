import { invoke } from '@tauri-apps/api/core';
import type { ExportPreview, SettingsInput, SourceInfo, Summary } from './types';

const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
const mockMode = typeof window !== 'undefined' && new URLSearchParams(window.location.search).has('mock');

const mockSource: SourceInfo = {
  path: '/mock/Milk-procurement.xlsx',
  fileName: 'Milk-procurement.xlsx',
  sheetName: 'Procurement',
  financialYear: '2025-26',
  reportingPeriod: '01/04/2025 to 31/03/2026',
  reportingMonths: 'APR, MAY, JUN, JUL, AUG, SEP, OCT, NOV, DEC, JAN, FEB, MAR',
  reportingMonthIndices: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
  dcmpu: 'ERODE',
  district: 'ERODE',
  society: 'ED 217 ODANILAI MPCS',
  societyCode: '15-10-00429',
  memberCount: 48
};

const mockSummary: Summary = {
  memberCount: 48,
  activeSubscriptions: 476,
  memberContribution: 3812,
  societyContribution: 476,
  unionContribution: 476,
  totalContribution: 4764
};

const desktopCommands = {
  importSource: (path: string) => invoke<SourceInfo>('import_source', { path }),
  removeSource: () => invoke<void>('remove_source'),
  summarize: (settings: SettingsInput) => invoke<Summary>('summarize', { settings }),
  previewExport: (settings: SettingsInput, destination: string) =>
    invoke<ExportPreview>('preview_export', { settings, destination }),
  exportWorkbook: (settings: SettingsInput, destination: string, overwriteExisting: boolean) =>
    invoke<string>('export_workbook', { settings, destination, overwriteExisting }),
  openOutput: (path: string) => invoke<void>('open_output', { path }),
  revealOutput: (path: string) => invoke<void>('reveal_output', { path })
};

const browserCommands = {
  importSource: async (path: string) => ({
    ...mockSource,
    path,
    fileName: path.split('/').pop() ?? mockSource.fileName
  }),
  removeSource: async () => undefined,
  summarize: async (settings: SettingsInput) => {
    if (!/^\d{4}-\d{2}$/.test(settings.financialYear)) throw new Error('Use a year like 2025-26.');
    return mockSummary;
  },
  previewExport: async (_settings: SettingsInput, destination: string) => ({
    destinationPath: destination,
    destinationExists: destination.startsWith('/mock/'),
    suggestedFileName: destination.split('/').pop() ?? 'FORM-10.xlsx'
  }),
  exportWorkbook: async (_settings: SettingsInput, destination: string) => destination,
  openOutput: async () => undefined,
  revealOutput: async () => undefined
};

export const commands = {
  ...(inTauri ? desktopCommands : mockMode ? browserCommands : desktopCommands)
};
