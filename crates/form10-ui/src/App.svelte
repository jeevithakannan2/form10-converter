<script lang="ts">
  import { onMount } from 'svelte';
  import { Calculator, CalendarDays, CircleAlert, FileOutput, FileSpreadsheet, Hash, Landmark, Moon, ReceiptText, Sun, X } from '@lucide/svelte';
  import ImportPanel from '$lib/components/ImportPanel.svelte';
  import SettingsForm from '$lib/components/SettingsForm.svelte';
  import WorkflowProgress from '$lib/components/WorkflowProgress.svelte';
  import { ConverterStore } from '$lib/state/converter.svelte';
  import { formatCurrency, fileName } from '$lib/utils/formatting';
  import { chooseOutputFile, chooseSourceFile, subscribeToFileDrops } from '$lib/api/bridge';

  const converter = new ConverterStore();
  let overwriteDestination = $state<string | null>(null);
  let overwriteFileName = $derived(overwriteDestination ? fileName(overwriteDestination) : '');
  let pointerFrame = 0;
  let targetPointerX = 0;
  let targetPointerY = 0;
  let renderedPointerX = 0;
  let renderedPointerY = 0;
  let pointerActive = $state(false);
  let appElement: HTMLElement;
  let backgroundSymbols = $state([
    { x: 9, y: 24, near: false, delay: '0s' }, { x: 24, y: 79, near: false, delay: '-4s' },
    { x: 38, y: 15, near: false, delay: '-9s' }, { x: 55, y: 73, near: false, delay: '-2s' },
    { x: 69, y: 27, near: false, delay: '-12s' }, { x: 82, y: 68, near: false, delay: '-6s' },
    { x: 93, y: 18, near: false, delay: '-15s' }
  ]);

  const supportedFile = (path: string) => /\.xlsx?$/i.test(path);

  async function chooseSource() {
    if (converter.busy) return;
    const path = await chooseSourceFile();
    if (typeof path === 'string') await converter.importSource(path);
  }

  async function chooseOutput() {
    if (!converter.readyToExport) {
      converter.notice = 'Complete the required details before exporting.';
      return;
    }
    converter.operation = 'choosing-output';
    try {
      const path = await chooseOutputFile(converter.settings.financialYear);
      if (typeof path !== 'string') return;
      converter.operation = 'idle';
      const preview = await converter.previewDestination(path);
      if (!preview) return;
      if (preview.destinationExists) {
        overwriteDestination = preview.destinationPath;
        return;
      }
      await converter.export(preview, false);
    } finally {
      if (converter.operation === 'choosing-output') converter.operation = 'idle';
    }
  }

  async function confirmOverwrite() {
    const destination = overwriteDestination;
    overwriteDestination = null;
    if (!destination) return;
    const preview = await converter.previewDestination(destination);
    if (preview) await converter.export(preview, true);
  }

  function changeSetting(key: keyof typeof converter.settings, value: string | number) {
    converter.changeSetting(key, value as never);
  }

  function trackPointer(event: PointerEvent) {
    targetPointerX = event.clientX;
    targetPointerY = event.clientY;
    if (!pointerActive) {
      renderedPointerX = targetPointerX;
      renderedPointerY = targetPointerY;
      pointerActive = true;
    }
    backgroundSymbols = backgroundSymbols.map((symbol) => ({
      ...symbol,
      near: Math.hypot(event.clientX - window.innerWidth * (symbol.x / 100), event.clientY - window.innerHeight * (symbol.y / 100)) < 150
    }));
  }

  function animatePointer() {
    renderedPointerX += (targetPointerX - renderedPointerX) * 0.12;
    renderedPointerY += (targetPointerY - renderedPointerY) * 0.12;
    appElement.style.setProperty('--pointer-x', `${renderedPointerX}px`);
    appElement.style.setProperty('--pointer-y', `${renderedPointerY}px`);
    pointerFrame = requestAnimationFrame(animatePointer);
  }

  onMount(() => {
    pointerFrame = requestAnimationFrame(animatePointer);
    const unsubscribe = subscribeToFileDrops((paths) => {
      if (converter.busy) return;
      const [path] = paths;
      if (!path) return;
      if (!supportedFile(path)) {
        converter.notice = 'Choose an .xls or .xlsx file.';
        return;
      }
      void converter.importSource(path);
    });
    return () => {
      cancelAnimationFrame(pointerFrame);
      unsubscribe();
    };
  });
</script>

<svelte:head><title>Form 10 Converter</title></svelte:head>

<main bind:this={appElement} class:dark={converter.appearance === 'dark'} class:pointer-active={pointerActive} onpointermove={trackPointer}>
  <div class="pointer-glow" aria-hidden="true"></div>
  <div class="background-symbols" aria-hidden="true">
    {#each backgroundSymbols as symbol, index}
      <span class:near={symbol.near} class="background-symbol" style={`--symbol-x: ${symbol.x}%; --symbol-y: ${symbol.y}%; --symbol-delay: ${symbol.delay};`}>
        {#if index === 0}<FileSpreadsheet size={24} />
        {:else if index === 1}<Calculator size={22} />
        {:else if index === 2}<Landmark size={23} />
        {:else if index === 3}<ReceiptText size={22} />
        {:else if index === 4}<CalendarDays size={23} />
        {:else if index === 5}<Hash size={22} />
        {:else}<FileOutput size={23} />{/if}
      </span>
    {/each}
  </div>
  <div class="shell">
    <header class="app-header">
      <div class="brand"><span class="brand-mark"><FileOutput size={22} /></span><div><h1>Form 10 Converter</h1><p>Excel to statutory workbook</p></div></div>
      <button class="icon-button appearance" type="button" onclick={() => converter.toggleAppearance()} aria-label="Toggle color appearance">
        {#if converter.appearance === 'light'}<Moon size={19} />{:else}<Sun size={19} />{/if}
      </button>
    </header>

    <WorkflowProgress stage={converter.stage} hasSource={converter.source !== null} hasOutput={converter.outputPath !== null} onSelect={(stage) => converter.setStage(stage)} />

    {#if converter.notice}
      <section class="notice" role="alert" aria-labelledby="notice-title">
        <span class="notice-icon"><CircleAlert size={20} strokeWidth={2} /></span>
        <div class="notice-copy"><span class="notice-kicker">Needs attention</span><strong id="notice-title">{converter.stage === 1 ? 'This workbook is not ready to import' : 'Check the details and try again'}</strong><p>{converter.notice}</p>{#if converter.stage === 1}<span>Choose the month-wise procurement workbook, then try again.</span>{/if}</div>
        <div class="notice-actions">{#if converter.stage === 1}<button class="notice-retry" type="button" onclick={chooseSource}>Choose another file</button>{/if}<button class="notice-close" type="button" onclick={() => converter.notice = null} aria-label="Dismiss message"><X size={17} /></button></div>
      </section>
    {/if}

    {#if converter.stage === 1}
      <ImportPanel source={converter.source} busy={converter.busy} draggingFile={converter.draggingFile} onChoose={chooseSource} onRemove={() => converter.removeSource()} />
      {#if converter.source}
        <div class="stage-actions stage-actions-end"><button class="primary" type="button" onclick={() => converter.setStage(2)}>Continue to details</button></div>
      {/if}
    {/if}

    {#if converter.source && converter.stage === 2}
      <SettingsForm settings={converter.settings} reportingMonthIndices={converter.source.reportingMonthIndices} disabled={converter.busy} useOldRates={converter.useOldRates} onChange={changeSetting} onUseOldRates={(value) => converter.setUseOldRates(value)} />
      <div class="stage-actions"><button class="secondary" type="button" onclick={() => converter.setStage(1)}>Back</button><button class="primary" type="button" onclick={() => converter.setStage(3)} disabled={!converter.summary || converter.busy}>Continue to export</button></div>
    {/if}

    {#if converter.source && converter.stage === 3}
      <section class="panel export-panel" aria-labelledby="export-title">
        <div class="section-heading"><span class="eyebrow">03 / Output</span><h2 id="export-title">Ready to create FORM-10</h2><p>Review the conversion summary, then choose a location for the workbook.</p></div>
        {#if converter.operation === 'exporting'}
          <div class="busy-card"><span class="loader"></span><div><strong>Creating FORM-10</strong><span>Writing the Excel workbook...</span></div></div>
        {:else if converter.outputPath}
          <div class="success-panel"><span class="success-mark">✓</span><div class="success-copy"><span class="eyebrow">Workbook created</span><h3>FORM-10 is ready</h3><strong>{fileName(converter.outputPath)}</strong><p>{converter.outputPath}</p></div><div class="success-actions"><button class="primary" type="button" onclick={() => converter.openOutput()}>Open file</button><button class="secondary" type="button" onclick={() => converter.openOutput(true)}>Show folder</button><button class="secondary" type="button" onclick={() => converter.startAgain()}>Convert another file</button></div></div>
        {:else}
          <div class="export-grid export-review">
            <div class="output-file"><FileOutput size={21} /><div><span>Output workbook</span><strong>FORM-10-{converter.settings.financialYear}.xlsx</strong><small>Excel workbook (.xlsx)</small></div></div>
            {#if converter.summary}
              <div class="metrics"><div><span>Members</span><strong>{converter.summary.memberCount}</strong></div><div><span>Active subscriptions</span><strong>{converter.summary.activeSubscriptions}</strong></div><div class="metric-total"><span>Total contributions</span><strong>{formatCurrency(converter.summary.totalContribution)}</strong></div></div>
            {:else}<p class="validation-copy">Complete valid details to calculate the export summary.</p>{/if}
            <div class="export-callout"><div><strong>Your source workbook will not be changed.</strong><span>A new FORM-10 workbook will be created in your selected location.</span></div><button class="primary export-button" type="button" onclick={chooseOutput} disabled={!converter.readyToExport}>{converter.operation === 'choosing-output' ? 'Choose location...' : 'Choose location and create'}</button></div>
          </div>
        {/if}
      </section>
      {#if !converter.outputPath}<div class="stage-actions"><button class="secondary" type="button" onclick={() => converter.setStage(2)}>Back to details</button></div>{/if}
    {/if}
  </div>

  {#if overwriteDestination}
    <div class="dialog-backdrop" role="presentation"><div class="confirm-dialog" role="alertdialog" aria-modal="true" aria-labelledby="overwrite-title"><span class="eyebrow">Existing file</span><h2 id="overwrite-title">Replace this workbook?</h2><p><strong>{overwriteFileName}</strong> already exists. Replacing it cannot be undone.</p><p class="path">{overwriteDestination}</p><div><button class="secondary" type="button" onclick={() => overwriteDestination = null}>Cancel</button><button class="danger" type="button" onclick={confirmOverwrite}>Replace file</button></div></div></div>
  {/if}
</main>
