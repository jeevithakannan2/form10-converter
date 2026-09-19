import { open as tauriOpen, save as tauriSave } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';

const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
const mockMode = typeof window !== 'undefined' && new URLSearchParams(window.location.search).has('mock');

export const isTauri = () => inTauri;

export async function chooseSourceFile() {
  if (inTauri) {
    return tauriOpen({
      title: 'Choose Excel file',
      multiple: false,
      filters: [{ name: 'Excel files', extensions: ['xls', 'xlsx'] }]
    });
  }
  return mockMode ? '/mock/Milk-procurement.xlsx' : null;
}

export async function chooseOutputFile(financialYear: string) {
  if (inTauri) {
    return tauriSave({
      title: 'Save FORM-10',
      defaultPath: `FORM-10-${financialYear}.xlsx`,
      filters: [{ name: 'Excel workbook', extensions: ['xlsx'] }]
    });
  }
  if (!mockMode) return null;
  return new URLSearchParams(window.location.search).has('existing-output')
    ? `/mock/FORM-10-${financialYear}.xlsx`
    : `/exports/FORM-10-${financialYear}.xlsx`;
}

export function subscribeToFileDrops(onDrop: (paths: string[]) => void) {
  if (!inTauri) return () => {};
  const unlisten = getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === 'drop') onDrop(event.payload.paths);
  });
  return () => {
    void unlisten.then((stop) => stop());
  };
}
