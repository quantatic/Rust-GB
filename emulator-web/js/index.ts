import {ButtonType, Emulator} from '../pkg';
import '../static/main.css';

const PIXEL_SCALE = 4;

let emulator: Emulator | null = null;

const fileUpload = document.getElementById('file-upload') as HTMLInputElement;
fileUpload.addEventListener('input', (e) => {
  const romFile = fileUpload.files![0];
  romFile.arrayBuffer().then((buf) => {
    emulator = new Emulator(new Uint8Array(buf));
  });
});

const handleKeyEvent = (key: string, pressed: boolean) => {
  if (emulator === null) {
    return;
  }

  switch (key) {
    case 'x':
      emulator.set_button_pressed(ButtonType.A, pressed);
      break;
    case 'z':
      emulator.set_button_pressed(ButtonType.B, pressed);
      break;
    case 'Enter':
      emulator.set_button_pressed(ButtonType.Start, pressed);
      break;
    case 'Shift':
      emulator.set_button_pressed(ButtonType.Select, pressed);
      break;
    case 'ArrowUp':
      emulator.set_button_pressed(ButtonType.Up, pressed);
      break;
    case 'ArrowDown':
      emulator.set_button_pressed(ButtonType.Down, pressed);
      break;
    case 'ArrowLeft':
      emulator.set_button_pressed(ButtonType.Left, pressed);
      break;
    case 'ArrowRight':
      emulator.set_button_pressed(ButtonType.Right, pressed);
      break;
    default:
      console.log(`don't know how to handle: ${key} pressed: ${pressed}`);
      break;
  }
};

document.addEventListener('keydown', (e) => {
  handleKeyEvent(e.key, true);
});

document.addEventListener('keyup', (e) => {
  handleKeyEvent(e.key, false);
});

const ppuWidth = Emulator.ppuWidth();
const ppuHeight = Emulator.ppuHeight();

const canvas = document.getElementById('render-canvas') as HTMLCanvasElement;

canvas.width = ppuWidth * PIXEL_SCALE;
canvas.height = ppuHeight * PIXEL_SCALE;

const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
ctx.fillStyle = 'red';
ctx.fillRect(50, 50, 50, 50);
ctx.fillStyle = 'green';
ctx.fillRect(80, 80, 50, 50);


const runTick = () => {
  if (emulator === null) {
    return;
  }

  for (let i = 0; i < 25_000; i++) {
    emulator.step();
  }

  const buffer = emulator.buffer();
  for (let y = 0; y < ppuHeight; y++) {
    for (let x = 0; x < ppuWidth; x++) {
      const offset = ((y * ppuWidth) + x) * 3;
      const [red, green, blue] = buffer.slice(offset, offset + 3);
      ctx.fillStyle = `rgb(${red}, ${green}, ${blue})`;
      ctx.fillRect(x * PIXEL_SCALE, y * PIXEL_SCALE, PIXEL_SCALE, PIXEL_SCALE);
    }
  }
};

const animationFrame = () => {
  runTick();
  requestAnimationFrame(animationFrame);
};

requestAnimationFrame(animationFrame);
