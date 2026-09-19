<script lang="ts">
  import type { SettingsInput } from '$lib/api/types';
  import { monthOptions } from '$lib/utils/months';
  import { CalendarRange, Landmark, ReceiptText } from 'lucide-svelte';

  let { settings, disabled, useOldRates, onChange, onUseOldRates }: { settings: SettingsInput; disabled: boolean; useOldRates: boolean; onChange: (key: keyof SettingsInput, value: string | number) => void; onUseOldRates: (value: boolean) => void } = $props();
  const update = (event: Event) => {
    const input = event.currentTarget as HTMLInputElement;
    onChange(input.name as keyof SettingsInput, input.value);
  };
  const selectMonth = (value: number) => onChange('newFromMonth', value);
  let rateStartMonths = $derived(monthOptions(settings.financialYear));
</script>

<section class="details" aria-labelledby="details-title">
  <div class="section-heading"><span class="eyebrow">02 / Details</span><h2 id="details-title">Set up this register</h2><p>Confirm the society identity, then define how monthly subscriptions are valued.</p></div>
  <div class="settings-grid">
    <section class="panel settings-panel"><div class="panel-title"><span class="section-icon"><Landmark size={18} /></span><div><h3>Society details</h3><p>Printed at the top of every FORM-10 register.</p></div></div><div class="field-grid">
      <label>Financial year<input name="financialYear" value={settings.financialYear} oninput={update} disabled={disabled} placeholder="2025-26" /></label>
      <label>DCMPU<input name="dcmpu" value={settings.dcmpu} oninput={update} disabled={disabled} /></label>
      <label>District<input name="district" value={settings.district} oninput={update} disabled={disabled} /></label>
      <label>Society<input name="society" value={settings.society} oninput={update} disabled={disabled} /></label>
      <label class="wide-field">Society code<input name="societyCode" value={settings.societyCode} oninput={update} disabled={disabled} /></label>
    </div></section>
    <section class="panel settings-panel"><div class="panel-title"><span class="section-icon"><ReceiptText size={18} /></span><div><h3>Contribution rates</h3><p>Amounts are charged for each month marked paid.</p></div></div><div class="rate-start"><label class="toggle-row"><input type="checkbox" checked={useOldRates} onchange={(event) => onUseOldRates((event.currentTarget as HTMLInputElement).checked)} disabled={disabled} /><span><strong>Use old rates for the full year</strong><small>Turn this off to choose the month when the new rates begin.</small></span></label><fieldset class="month-picker" disabled={disabled || useOldRates}><legend><CalendarRange size={15} /> New rates begin in</legend><div class="month-grid">{#each rateStartMonths as option}<button class:chosen={settings.newFromMonth === option.value} type="button" onclick={() => selectMonth(option.value)} aria-pressed={settings.newFromMonth === option.value}>{option.short}</button>{/each}</div><p>{useOldRates ? 'Old rates apply from April through March.' : `New rates apply from ${rateStartMonths.find((option) => option.value === settings.newFromMonth)?.label ?? 'April'} through March.`}</p></fieldset></div><div class="rate-columns"><div><span>Old rates</span><label>Member<input name="oldMember" value={settings.oldMember} oninput={update} disabled={disabled} inputmode="decimal" /></label><label>Society<input name="oldSociety" value={settings.oldSociety} oninput={update} disabled={disabled} inputmode="decimal" /></label><label>Union<input name="oldUnion" value={settings.oldUnion} oninput={update} disabled={disabled} inputmode="decimal" /></label></div><div class:muted={useOldRates}><span>New rates</span><label>Member<input name="newMember" value={settings.newMember} oninput={update} disabled={disabled || useOldRates} inputmode="decimal" /></label><label>Society<input name="newSociety" value={settings.newSociety} oninput={update} disabled={disabled || useOldRates} inputmode="decimal" /></label><label>Union<input name="newUnion" value={settings.newUnion} oninput={update} disabled={disabled || useOldRates} inputmode="decimal" /></label></div></div></section>
  </div>
</section>
