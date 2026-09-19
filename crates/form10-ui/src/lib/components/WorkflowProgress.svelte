<script lang="ts">
  import { Check } from 'lucide-svelte';

  let { stage, hasSource, hasOutput, onSelect }: { stage: number; hasSource: boolean; hasOutput: boolean; onSelect: (stage: 1 | 2 | 3) => void } = $props();
  const steps = ['Import workbook', 'Review details', 'Export FORM-10'];
</script>

<ol class="progress" aria-label="Conversion progress">
  {#each steps as step, index}
    {@const stepNumber = (index + 1) as 1 | 2 | 3}
    {@const complete = index === 0 ? hasSource && stage > 1 : index === 1 ? stage > 2 || hasOutput : hasOutput}
    <li class:complete class:active={stage === stepNumber}>
      <button type="button" onclick={() => onSelect(stepNumber)} disabled={stepNumber > 1 && !hasSource} aria-current={stage === stepNumber ? 'step' : undefined}>
      <span class="step-number">{#if complete}<Check size={14} strokeWidth={3} />{:else}{index + 1}{/if}</span>
      <span>{step}</span>
      </button>
    </li>
  {/each}
</ol>
