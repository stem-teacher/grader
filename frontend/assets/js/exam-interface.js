// Chemical equation editor with real-time LaTeX preview
class ChemicalEditor {
  constructor(containerId, options = {}) {
    this.containerId = containerId;
    this.options = {
      height: options.height || '200px',
      placeholder: options.placeholder || 'Enter chemical equations using LaTeX notation...',
      ...options
    };
    this.init();
  }

  init() {
    const container = document.getElementById(this.containerId);
    if (!container) return;

    container.innerHTML = `
      <div class="chemical-editor-wrapper">
        <label>${this.options.label || 'Chemical Equation:'}</label>
        <textarea 
          id="${this.containerId}_input" 
          class="chemical-editor"
          style="height: ${this.options.height}; width: 100%;"
          placeholder="${this.options.placeholder}"
        ></textarea>
        <div class="latex-preview" id="${this.containerId}_preview">
          <strong>Preview:</strong>
          <div id="${this.containerId}_rendered"></div>
        </div>
      </div>
    `;

    const input = document.getElementById(`${this.containerId}_input`);
    const preview = document.getElementById(`${this.containerId}_rendered`);

    // Real-time LaTeX rendering
    input.addEventListener('input', () => {
      this.renderLatex(input.value, preview);
    });

    // Initialize MathJax configuration
    this.configureMathJax();
  }

  configureMathJax() {
    if (window.MathJax) {
      window.MathJax.startup.ready();
    }
  }

  renderLatex(text, previewElement) {
    if (!text.trim()) {
      previewElement.innerHTML = '<em>Enter equations to see preview</em>';
      return;
    }

    // Convert common chemistry notation to MathJax
    const processedText = this.processChemicalNotation(text);
    
    previewElement.innerHTML = `\\[${processedText}\\]`;
    
    if (window.MathJax) {
      MathJax.typesetPromise([previewElement]).catch((err) => {
        previewElement.innerHTML = '<em style="color: red;">Error in equation syntax</em>';
      });
    }
  }

  processChemicalNotation(text) {
    // Convert common chemical notation to LaTeX
    return text
      .replace(/\b(\d+)\b/g, '_{$1}')  // Subscripts for numbers
      .replace(/\^(\d+[+-]?)/g, '^{$1}')  // Superscripts for charges
      .replace(/\s*->\s*/g, ' \\rightarrow ')  // Reaction arrows
      .replace(/\s*<->\s*/g, ' \\leftrightarrow ')  // Equilibrium arrows
      .replace(/\s*\+\s*/g, ' + ')  // Plus signs
      .replace(/\(aq\)/g, '_{(aq)}')  // Aqueous state
      .replace(/\(s\)/g, '_{(s)}')   // Solid state
      .replace(/\(l\)/g, '_{(l)}')   // Liquid state
      .replace(/\(g\)/g, '_{(g)}');  // Gas state
  }

  getValue() {
    const input = document.getElementById(`${this.containerId}_input`);
    return input ? input.value : '';
  }
}

// JSME Chemical Structure Editor Integration
class StructureEditor {
  constructor() {
    this.jsmeApplet = null;
    this.init();
  }

  init() {
    if (typeof JSApplet !== 'undefined') {
      this.jsmeApplet = new JSApplet.JSME('jsme-container', '100%', '300px', {
        options: 'query,hydrogens'
      });
    }
  }

  getSmiles() {
    return this.jsmeApplet ? this.jsmeApplet.smiles() : '';
  }

  getMolFile() {
    return this.jsmeApplet ? this.jsmeApplet.molFile() : '';
  }

  clear() {
    if (this.jsmeApplet) {
      this.jsmeApplet.clear();
    }
  }
}

// Initialize editors when document is ready
document.addEventListener('DOMContentLoaded', function() {
  // Initialize all chemical editors
  const chemicalEditors = document.querySelectorAll('[id$="_editor"]');
  chemicalEditors.forEach(editor => {
    new ChemicalEditor(editor.id);
  });

  // Initialize structure editor
  const structureEditor = new StructureEditor();

  // Handle structure insertion
  document.getElementById('insert_structure')?.addEventListener('click', function() {
    const smiles = structureEditor.getSmiles();
    if (smiles) {
      // Insert SMILES into currently focused text area
      const activeElement = document.activeElement;
      if (activeElement && activeElement.tagName === 'TEXTAREA') {
        const cursorPos = activeElement.selectionStart;
        const textBefore = activeElement.value.substring(0, cursorPos);
        const textAfter = activeElement.value.substring(activeElement.selectionEnd);
        activeElement.value = textBefore + `\\ce{${smiles}}` + textAfter;
        
        // Trigger input event for real-time preview
        activeElement.dispatchEvent(new Event('input'));
      }
    }
  });

  // Handle structure clearing
  document.getElementById('clear_structure')?.addEventListener('click', function() {
    structureEditor.clear();
  });
});

// Utility functions for Shiny integration
function chemical_editor_ui(id, label, height = '200px') {
  return `<div id="${id}_editor" data-label="${label}" data-height="${height}"></div>`;
}

function calculation_editor_ui(id, label, height = '300px') {
  return chemical_editor_ui(id, label, height);
}
