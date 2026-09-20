import { commands } from '$lib/api/commands';
import { isTauri } from '$lib/api/bridge';
import type { ExportPreview, SettingsInput, SourceInfo, Summary } from '$lib/api/types';

export type Operation = 'idle' | 'importing' | 'choosing-output' | 'exporting';
export type Stage = 1 | 2 | 3;

const defaults: SettingsInput = {
  financialYear: '',
  dcmpu: 'ERODE',
  district: 'ERODE',
  society: '',
  societyCode: '',
  oldMember: '10',
  oldSociety: '1',
  oldUnion: '1',
  newMember: '10',
  newSociety: '1',
  newUnion: '1',
  newFromMonth: 0
};

export class ConverterStore {
  appearance = $state<'light' | 'dark'>('light');
  operation = $state<Operation>('idle');
  draggingFile = $state(false);
  source = $state<SourceInfo | null>(null);
  summary = $state<Summary | null>(null);
  outputPath = $state<string | null>(null);
  notice = $state<string | null>(null);
  stage = $state<Stage>(1);
  settings = $state<SettingsInput>({ ...defaults });
  private validationTimer: ReturnType<typeof setTimeout> | undefined;
  private validationVersion = 0;

  get busy() {
    return this.operation !== 'idle';
  }

  get readyToExport() {
    return this.source !== null && this.summary !== null && !this.busy;
  }

  get useOldRates() {
    return this.settings.newFromMonth === 0;
  }

  setStage(stage: Stage) {
    if (stage > 1 && !this.source) return;
    if (stage === 3 && !this.summary) {
      this.notice = 'Complete the required details before continuing.';
      return;
    }
    this.stage = stage;
  }

  setUseOldRates(useOldRates: boolean) {
    this.changeSetting('newFromMonth', useOldRates ? 0 : this.source?.reportingMonthIndices[0] ?? 1);
  }

  toggleAppearance() {
    this.appearance = this.appearance === 'light' ? 'dark' : 'light';
  }

  changeSetting<Key extends keyof SettingsInput>(key: Key, value: SettingsInput[Key]) {
    this.settings[key] = value;
    this.notice = null;
    this.outputPath = null;
    this.summary = null;
    this.scheduleSummary();
  }

  async importSource(path: string) {
    if (this.busy) return;
    this.operation = 'importing';
    this.notice = null;
    this.outputPath = null;
    this.summary = null;
    try {
      const source = await commands.importSource(path);
      this.source = source;
      if (source.financialYear) this.settings.financialYear = source.financialYear;
      if (source.dcmpu) this.settings.dcmpu = source.dcmpu;
      if (source.district) this.settings.district = source.district;
      if (source.society) this.settings.society = source.society;
      if (source.societyCode) this.settings.societyCode = source.societyCode;
      await this.refreshSummary();
      if (!isTauri()) {
        this.notice = 'Browser preview mode uses sample conversion data. Run the Tauri desktop app to validate and convert this workbook.';
      }
    } catch (error) {
      this.source = null;
      this.notice = message(error);
    } finally {
      this.operation = 'idle';
    }
  }

  async removeSource() {
    if (this.busy) return;
    try {
      await commands.removeSource();
      this.source = null;
      this.summary = null;
      this.outputPath = null;
      this.notice = null;
      this.stage = 1;
    } catch (error) {
      this.notice = message(error);
    }
  }

  async refreshSummary() {
    if (!this.source) return;
    const version = ++this.validationVersion;
    try {
      const summary = await commands.summarize({ ...this.settings });
      if (version === this.validationVersion) this.summary = summary;
    } catch (error) {
      if (version === this.validationVersion) {
        this.summary = null;
        this.notice = message(error);
      }
    }
  }

  scheduleSummary() {
    if (!this.source) return;
    if (this.validationTimer) clearTimeout(this.validationTimer);
    const version = ++this.validationVersion;
    this.validationTimer = setTimeout(async () => {
      try {
        const summary = await commands.summarize({ ...this.settings });
        if (version === this.validationVersion) this.summary = summary;
      } catch (error) {
        if (version === this.validationVersion) {
          this.summary = null;
          this.notice = message(error);
        }
      }
    }, 250);
  }

  async previewDestination(destination: string): Promise<ExportPreview | null> {
    if (!this.source || !this.summary || this.operation === 'importing' || this.operation === 'exporting') {
      if (!this.summary) this.notice = 'Complete the required details before exporting.';
      return null;
    }
    try {
      return await commands.previewExport({ ...this.settings }, destination);
    } catch (error) {
      this.notice = message(error);
      return null;
    }
  }

  async export(preview: ExportPreview, overwriteExisting: boolean) {
    if (this.busy) return;
    this.operation = 'exporting';
    this.notice = null;
    try {
      this.outputPath = await commands.exportWorkbook(
        { ...this.settings },
        preview.destinationPath,
        overwriteExisting
      );
    } catch (error) {
      this.notice = message(error);
    } finally {
      this.operation = 'idle';
    }
  }

  async openOutput(reveal = false) {
    if (!this.outputPath) return;
    try {
      if (reveal) await commands.revealOutput(this.outputPath);
      else await commands.openOutput(this.outputPath);
    } catch (error) {
      this.notice = message(error);
    }
  }

  async startAgain() {
    await this.removeSource();
  }
}

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));
