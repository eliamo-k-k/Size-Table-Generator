import { mkdir, readDir, writeFile } from "@tauri-apps/plugin-fs";
import html2canvas from "html2canvas";

export async function saveElementToPath(elementId: string, dest: string) {
  const canvas = await html2canvas(document.querySelector(`#${elementId}`)!);
  canvas.toBlob(async (blob) => {
    const buf = await blob?.arrayBuffer();
    await writeFile(dest, new Uint8Array(buf!));
  });
}

// if dir exists return true, otherwise create the dir return false
export async function checkDirThenCreate(dir: string) {
  try {
    await readDir(dir);
    return true;
  } catch {
    await mkdir(dir, { recursive: true });
    return false;
  }
}
