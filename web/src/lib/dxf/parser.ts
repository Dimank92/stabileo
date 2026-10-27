// Parse DXF text into a flat intermediate representation
// Uses dxf-parser library for the heavy lifting

import DxfParser from 'dxf-parser';
import type { ILineEntity } from 'dxf-parser';
import type { ILwpolylineEntity } from 'dxf-parser';
import type { IPointEntity } from 'dxf-parser';
import type { IInsertEntity } from 'dxf-parser';
import type { ITextEntity } from 'dxf-parser';
import type { IMtextEntity } from 'dxf-parser';
import type { ICircleEntity } from 'dxf-parser';
import type { DxfParseResult } from './types';

export function parseDxf(text: string): DxfParseResult {
  const parser = new DxfParser();
  let dxf;
  try {
    dxf = parser.parseSync(text);
  } catch {
    return { lines: [], points: [], inserts: [], texts: [], circles: [], layers: [] };
  }
  if (!dxf) {
    return { lines: [], points: [], inserts: [], texts: [], circles: [], layers: [] };
  }

  const result: DxfParseResult = {
    lines: [],
    points: [],
    inserts: [],
    texts: [],
    circles: [],
    layers: Object.keys(dxf.tables?.layer?.layers ?? {}),
  };

  for (const entity of dxf.entities) {
    const layer = (entity.layer ?? '0').toUpperCase();

    switch (entity.type) {
      case 'LINE': {
        const e = entity as ILineEntity;
        if (e.vertices && e.vertices.length >= 2) {
          result.lines.push({
            layer,
            start: {
              x: e.vertices[0].x,
              y: e.vertices[0].y,
              ...(e.vertices[0].z !== undefined && Math.abs(e.vertices[0].z) > 1e-9 ? { z: e.vertices[0].z } : {}),
            },
            end: {
              x: e.vertices[1].x,
              y: e.vertices[1].y,
              ...(e.vertices[1].z !== undefined && Math.abs(e.vertices[1].z) > 1e-9 ? { z: e.vertices[1].z } : {}),
            },
          });
        }
        break;
      }
      case 'LWPOLYLINE':
      case 'POLYLINE': {
        const e = entity as ILwpolylineEntity;
        if (e.vertices) {
          for (let i = 0; i < e.vertices.length - 1; i++) {
            result.lines.push({
              layer,
              start: {
                x: e.vertices[i].x,
                y: e.vertices[i].y,
                ...(e.vertices[i].z !== undefined && Math.abs(e.vertices[i].z) > 1e-9 ? { z: e.vertices[i].z } : {}),
              },
              end: {
                x: e.vertices[i + 1].x,
                y: e.vertices[i + 1].y,
                ...(e.vertices[i + 1].z !== undefined && Math.abs(e.vertices[i + 1].z) > 1e-9 ? { z: e.vertices[i + 1].z } : {}),
              },
            });
          }
          // Close if shape flag is set
          if (e.shape && e.vertices.length >= 3) {
            const last = e.vertices[e.vertices.length - 1];
            const first = e.vertices[0];
            result.lines.push({
              layer,
              start: {
                x: last.x,
                y: last.y,
                ...(last.z !== undefined && Math.abs(last.z) > 1e-9 ? { z: last.z } : {}),
              },
              end: {
                x: first.x,
                y: first.y,
                ...(first.z !== undefined && Math.abs(first.z) > 1e-9 ? { z: first.z } : {}),
              },
            });
          }
        }
        break;
      }
      case '3DFACE': {
        const e = entity as any;
        if (e.vertices && e.vertices.length >= 3) {
          const v = e.vertices;
          for (let i = 0; i < v.length; i++) {
            const next = (i + 1) % v.length;
            if (v[i].x === v[next].x && v[i].y === v[next].y && (v[i].z ?? 0) === (v[next].z ?? 0)) continue;
            result.lines.push({
              layer,
              start: {
                x: v[i].x,
                y: v[i].y,
                ...(v[i].z !== undefined && Math.abs(v[i].z) > 1e-9 ? { z: v[i].z } : {}),
              },
              end: {
                x: v[next].x,
                y: v[next].y,
                ...(v[next].z !== undefined && Math.abs(v[next].z) > 1e-9 ? { z: v[next].z } : {}),
              },
            });
          }
        }
        break;
      }
      case 'POINT': {
        const e = entity as IPointEntity;
        result.points.push({
          layer,
          position: {
            x: e.position.x,
            y: e.position.y,
            ...(e.position.z !== undefined && Math.abs(e.position.z) > 1e-9 ? { z: e.position.z } : {}),
          },
        });
        break;
      }
      case 'INSERT': {
        const e = entity as IInsertEntity;
        result.inserts.push({
          layer,
          position: {
            x: e.position.x,
            y: e.position.y,
            ...(e.position.z !== undefined && Math.abs(e.position.z) > 1e-9 ? { z: e.position.z } : {}),
          },
          blockName: e.name ?? '',
        });
        break;
      }
      case 'TEXT': {
        const e = entity as ITextEntity;
        const pos = e.startPoint ?? (e as any).position;
        if (pos) {
          result.texts.push({
            layer,
            position: {
              x: pos.x,
              y: pos.y,
              ...(pos.z !== undefined && Math.abs(pos.z) > 1e-9 ? { z: pos.z } : {}),
            },
            value: e.text ?? '',
          });
        }
        break;
      }
      case 'MTEXT': {
        const e = entity as IMtextEntity;
        if (e.position) {
          result.texts.push({
            layer,
            position: {
              x: e.position.x,
              y: e.position.y,
              ...(e.position.z !== undefined && Math.abs(e.position.z) > 1e-9 ? { z: e.position.z } : {}),
            },
            value: e.text ?? '',
          });
        }
        break;
      }
      case 'CIRCLE': {
        const e = entity as ICircleEntity;
        result.circles.push({
          layer,
          center: {
            x: e.center.x,
            y: e.center.y,
            ...(e.center.z !== undefined && Math.abs(e.center.z) > 1e-9 ? { z: e.center.z } : {}),
          },
          radius: e.radius,
        });
        break;
      }
    }
  }

  return result;
}
