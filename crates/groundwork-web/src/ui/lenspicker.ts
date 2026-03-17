/**
 * X-ray lens picker — replaces the simple Q on/off toggle with a multi-lens system.
 *
 * Each lens visualizes a different underground/data layer with its own color theme:
 * - Roots (default): transparent ground, visible root networks
 * - Irrigation: 3D moisture heatmap — blue (wet) to red (dry)
 * - (Future: Nutrients, Light, Soil, etc.)
 *
 * The lens picker appears when x-ray mode is activated (Q key or button).
 * See decisions/2026-03-17T18:00:00_reduce_progression_intensity.md
 */

export type XrayLens = 'off' | 'roots' | 'irrigation';

interface LensDef {
  id: XrayLens;
  name: string;
  description: string;
  color: string; // CSS color for the lens indicator
}

const LENS_DEFS: LensDef[] = [
  { id: 'roots', name: 'Roots', description: 'See root networks underground', color: '#88cc66' },
  { id: 'irrigation', name: 'Irrigation', description: 'Moisture heatmap — blue=wet, red=dry', color: '#4488cc' },
];

export class LensPicker {
  private el: HTMLElement;
  private _lens: XrayLens = 'off';
  private _visible = false;
  private _onChange: ((lens: XrayLens) => void) | null = null;
  private _onSelectIrrigation: (() => void) | null = null;

  constructor() {
    this.el = document.createElement('div');
    this.el.id = 'lens-picker';
    document.body.appendChild(this.el);

    const style = document.createElement('style');
    style.textContent = LENS_CSS;
    document.head.appendChild(style);

    this.buildUI();
  }

  get lens(): XrayLens { return this._lens; }
  get visible(): boolean { return this._visible; }

  /** Register callback for lens changes */
  onChange(cb: (lens: XrayLens) => void): void {
    this._onChange = cb;
  }

  /** Register callback for irrigation lens selection (quest tracking) */
  onSelectIrrigation(cb: () => void): void {
    this._onSelectIrrigation = cb;
  }

  /** Show the lens picker (called when x-ray activates) */
  show(): void {
    this._visible = true;
    this.el.classList.add('visible');
    this.updateButtons();
  }

  /** Hide the lens picker (called when x-ray deactivates) */
  hide(): void {
    this._visible = false;
    this.el.classList.remove('visible');
  }

  /** Set lens directly */
  setLens(lens: XrayLens): void {
    this._lens = lens;
    if (lens === 'off') {
      this.hide();
    } else {
      this.show();
    }
    this.updateButtons();
    this._onChange?.(lens);
    if (lens === 'irrigation') {
      this._onSelectIrrigation?.();
    }
  }

  /** Toggle x-ray on/off. If turning on, defaults to 'roots'. */
  toggle(): XrayLens {
    if (this._lens === 'off') {
      this.setLens('roots');
    } else {
      this.setLens('off');
    }
    return this._lens;
  }

  /** Cycle through lenses (when already in x-ray) */
  cycle(): XrayLens {
    const lensIds = LENS_DEFS.map(l => l.id);
    const idx = lensIds.indexOf(this._lens);
    const next = (idx + 1) % lensIds.length;
    this.setLens(lensIds[next]);
    return this._lens;
  }

  private buildUI(): void {
    let html = '<div class="lens-label">X-Ray Lens</div><div class="lens-buttons">';
    for (const def of LENS_DEFS) {
      html += `<button class="lens-btn" data-lens="${def.id}" title="${def.description}">
        <span class="lens-dot" style="background:${def.color}"></span>
        <span class="lens-name">${def.name}</span>
      </button>`;
    }
    html += '</div>';
    this.el.innerHTML = html;

    // Wire up buttons
    for (const btn of this.el.querySelectorAll('.lens-btn')) {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const lens = (btn as HTMLElement).dataset.lens as XrayLens;
        this.setLens(lens);
      });
    }
  }

  private updateButtons(): void {
    for (const btn of this.el.querySelectorAll('.lens-btn')) {
      const lens = (btn as HTMLElement).dataset.lens;
      btn.classList.toggle('active', lens === this._lens);
    }
  }
}

const LENS_CSS = `
#lens-picker {
  position: absolute;
  top: 50%;
  left: 12px;
  transform: translateY(-50%) translateX(-8px);
  background: rgba(20, 18, 15, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 10px 12px;
  backdrop-filter: blur(8px);
  font-family: system-ui, -apple-system, sans-serif;
  color: #d4c8a8;
  z-index: 15;
  pointer-events: auto;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.25s ease, visibility 0.25s ease, transform 0.25s ease;
}
#lens-picker.visible {
  opacity: 1;
  visibility: visible;
  transform: translateY(-50%) translateX(0);
}

.lens-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: rgba(200, 180, 140, 0.5);
  margin-bottom: 8px;
}

.lens-buttons {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.lens-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.05);
  color: #b8a88a;
  cursor: pointer;
  font-size: 12px;
  font-family: inherit;
  transition: all 0.12s ease;
  white-space: nowrap;
}
.lens-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
}
.lens-btn.active {
  background: rgba(80, 120, 60, 0.5);
  border-color: rgba(140, 190, 80, 0.5);
  color: #c8e8a0;
}

.lens-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.lens-name {
  font-weight: 500;
}

@media (max-width: 768px) {
  #lens-picker {
    left: 6px;
    padding: 8px 10px;
  }
  .lens-btn {
    padding: 8px 10px;
    min-height: 36px;
    font-size: 13px;
  }
}
`;
