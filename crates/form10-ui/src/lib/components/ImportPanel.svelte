<script lang="ts">
  import { FileSpreadsheet, FolderOpen, LoaderCircle, X } from 'lucide-svelte';
  import type { SourceInfo } from '$lib/api/types';

  let {
    source,
    busy,
    draggingFile,
    onChoose,
    onRemove
  }: {
    source: SourceInfo | null;
    busy: boolean;
    draggingFile: boolean;
    onChoose: () => void;
    onRemove: () => void;
  } = $props();
</script>

<section class="panel import-panel" aria-labelledby="import-title">
  <div class="section-heading"><span class="eyebrow">01 / Source</span><h2 id="import-title">Import workbook</h2><p>Select the month-wise Excel file to begin. It stays on this device.</p></div>
  {#if busy && !source}
    <div class="busy-card"><LoaderCircle class="spin" size={24} /><div><strong>Reading workbook</strong><span>Checking columns and members...</span></div></div>
  {:else if source}
    <div class="source-row">
      <span class="file-icon"><FileSpreadsheet size={26} /></span>
      <div class="source-copy"><strong>{source.fileName}</strong><span>Excel workbook · ready to convert</span></div>
      <span class="status">Ready</span>
      <button class="icon-button" type="button" onclick={onRemove} disabled={busy} aria-label="Remove workbook"><X size={18} /></button>
    </div>
    <dl class="source-stats"><div><dt>Sheet</dt><dd>{source.sheetName}</dd></div><div><dt>Year</dt><dd>{source.financialYear ?? 'Not detected'}</dd></div><div><dt>Members</dt><dd>{source.memberCount}</dd></div></dl>
    <button class="text-action" type="button" onclick={onChoose} disabled={busy}>Replace workbook</button>
  {:else}
    <button class:dragging={draggingFile} class="drop-zone" type="button" onclick={onChoose} disabled={busy}>
      <span class="drop-icon"><FolderOpen size={27} /></span>
      <strong>{draggingFile ? 'Release to import' : 'Drop your workbook here'}</strong>
      <span>{draggingFile ? 'We will validate it before continuing' : 'or select it from your computer'}</span>
      <span class="button-like">Choose Excel file</span>
      <small>Excel .xls or .xlsx</small>
    </button>
  {/if}
</section>
