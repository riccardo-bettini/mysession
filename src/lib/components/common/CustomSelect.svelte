<script lang="ts">
  import { onMount } from "svelte";

  export interface SelectOption<T = any> {
    value: T;
    label: string;
    description?: string;
    badge?: string;
  }

  interface Props<T = any> {
    value?: T;
    options: SelectOption<T>[];
    placeholder?: string;
    disabled?: boolean;
    ariaLabel?: string;
    onchange?: (val: T) => void;
  }

  let {
    value = $bindable(),
    options = [],
    placeholder = "Seleziona…",
    disabled = false,
    ariaLabel,
    onchange
  }: Props = $props();

  let isOpen = $state(false);
  let containerEl = $state<HTMLDivElement | null>(null);

  const selectedOption = $derived(
    options.find((opt) => opt.value === value) ?? null
  );

  function toggleOpen() {
    if (disabled) return;
    isOpen = !isOpen;
  }

  function handleSelect(val: any) {
    value = val;
    isOpen = false;
    onchange?.(val);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (disabled) return;

    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      isOpen = !isOpen;
    } else if (e.key === "Escape" && isOpen) {
      e.preventDefault();
      isOpen = false;
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!isOpen) {
        isOpen = true;
      } else {
        const curIdx = options.findIndex((o) => o.value === value);
        const nextIdx = curIdx < options.length - 1 ? curIdx + 1 : 0;
        handleSelect(options[nextIdx].value);
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!isOpen) {
        isOpen = true;
      } else {
        const curIdx = options.findIndex((o) => o.value === value);
        const prevIdx = curIdx > 0 ? curIdx - 1 : options.length - 1;
        handleSelect(options[prevIdx].value);
      }
    }
  }

  onMount(() => {
    function handleGlobalClick(e: MouseEvent) {
      if (!isOpen) return;
      const target = e.target as HTMLElement | null;
      if (!target || !containerEl?.contains(target)) {
        isOpen = false;
      }
    }

    window.addEventListener("pointerdown", handleGlobalClick);
    return () => {
      window.removeEventListener("pointerdown", handleGlobalClick);
    };
  });
</script>

<div
  bind:this={containerEl}
  class="custom-select-container"
  class:is-open={isOpen}
  class:is-disabled={disabled}
>
  <button
    type="button"
    class="select-trigger pressable"
    onclick={toggleOpen}
    onkeydown={handleKeyDown}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    aria-label={ariaLabel || selectedOption?.label || placeholder}
    {disabled}
  >
    <span class="trigger-label" class:is-placeholder={!selectedOption}>
      {selectedOption ? selectedOption.label : placeholder}
    </span>
    <svg
      class="select-chevron"
      class:rotated={isOpen}
      viewBox="0 0 16 16"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M4.5 6.25L8 9.75L11.5 6.25"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  {#if isOpen}
    <div
      class="select-popover"
      role="listbox"
      aria-label={ariaLabel || "Opzioni di selezione"}
      tabindex="-1"
    >
      <div class="popover-options-list">
        {#each options as opt (opt.value)}
          {@const isSelected = opt.value === value}
          <button
            type="button"
            role="option"
            aria-selected={isSelected}
            class="option-row pressable"
            class:active={isSelected}
            onclick={() => handleSelect(opt.value)}
          >
            <div class="option-content">
              <span class="option-label">{opt.label}</span>
              {#if opt.description}
                <span class="option-desc">{opt.description}</span>
              {/if}
            </div>

            <div class="option-trailing">
              {#if opt.badge}
                <span class="option-badge">{opt.badge}</span>
              {/if}
              {#if isSelected}
                <svg
                  class="check-icon"
                  viewBox="0 0 16 16"
                  fill="none"
                  aria-hidden="true"
                >
                  <path
                    d="M3.75 8.25L6.5 11L12.25 5.25"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .custom-select-container {
    position: relative;
    width: 100%;
    user-select: none;
    -webkit-user-select: none;
  }

  .select-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.65rem;
    padding: 0.54rem 0.75rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-weight: 500;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 140ms ease,
      box-shadow 140ms ease,
      background-color 140ms ease,
      transform 120ms var(--spring-press);
  }

  .select-trigger:hover:not(:disabled) {
    background: var(--surface-pill-hover);
    border-color: var(--hairline-border-strongest, rgba(255, 255, 255, 0.25));
  }

  .select-trigger:focus-visible {
    outline: none;
    border-color: var(--accent-primary, #007aff);
    box-shadow: 0 0 0 2.5px var(--accent-soft);
  }

  .custom-select-container.is-open .select-trigger {
    border-color: var(--accent-primary, #007aff);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .custom-select-container.is-disabled .select-trigger {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .trigger-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .trigger-label.is-placeholder {
    color: var(--text-tertiary);
  }

  .select-chevron {
    width: 0.85rem;
    height: 0.85rem;
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: transform 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .select-chevron.rotated {
    transform: rotate(180deg);
  }

  /* ==========================================================================
     Floating Popover (Apple HIG Vibrancy)
     ========================================================================== */
  .select-popover {
    position: absolute;
    top: calc(100% + 0.35rem);
    left: 0;
    right: 0;
    z-index: 400;
    padding: 0.35rem;
    border-radius: var(--radius-lg);
    background: var(--popover-bg);
    backdrop-filter: blur(28px) saturate(180%);
    -webkit-backdrop-filter: blur(28px) saturate(180%);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow: var(--shadow-popover);
    animation: popoverEnter 140ms cubic-bezier(0.16, 1, 0.3, 1);
    max-height: 16rem;
    overflow-y: auto;
  }

  @keyframes popoverEnter {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .popover-options-list {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .option-row {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.48rem 0.65rem;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    transition: background-color 100ms ease, color 100ms ease;
  }

  .option-row:hover {
    background: var(--surface-pill-hover);
  }

  .option-row.active {
    background: var(--accent-soft);
    color: var(--accent-primary, #007aff);
    font-weight: 600;
  }

  .option-content {
    display: flex;
    flex-direction: column;
    gap: 0.12rem;
    min-width: 0;
    flex: 1;
  }

  .option-label {
    font-size: 0.8rem;
    letter-spacing: var(--tracking-normal);
  }

  .option-desc {
    font-size: 0.68rem;
    color: var(--text-secondary);
    opacity: 0.85;
  }

  .option-trailing {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .option-badge {
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-pill);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    color: var(--text-secondary);
  }

  .check-icon {
    width: 0.85rem;
    height: 0.85rem;
    color: var(--accent-primary, #007aff);
  }

  .pressable:active {
    transform: scale(0.985);
  }
</style>
