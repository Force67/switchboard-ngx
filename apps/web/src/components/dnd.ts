import type { DragState, ID } from "./sidebarTypes";

export interface DragCallbacks {
  onDragStart: (kind: "chat"|"folder", id: ID, fromFolderId?: ID) => void;
  onDragMove: (over: DragState["over"]) => void;
  onDragEnd: (target: DragState["over"]) => void;
  onAutoExpand: (folderId: ID) => void;
}

export class DragManager {
  private callbacks: DragCallbacks;
  private dragState: DragState | null = null;
  private ghostElement: HTMLElement | null = null;
  private dropIndicator: HTMLElement | null = null;
  private autoExpandTimer: number | null = null;
  private startX = 0;
  private startY = 0;
  private dragThreshold = 4;
  private dragDelay = 120;

  constructor(callbacks: DragCallbacks) {
    this.callbacks = callbacks;
  }

  startDrag(element: HTMLElement, kind: "chat"|"folder", id: ID, fromFolderId?: ID) {
    element.addEventListener("pointerdown", (e) => this.handlePointerDown(e, element, kind, id, fromFolderId));
  }

  private handlePointerDown(e: PointerEvent, element: HTMLElement, kind: "chat"|"folder", id: ID, fromFolderId?: ID) {
    if (e.button !== 0) return; // Only left mouse button

    this.startX = e.clientX;
    this.startY = e.clientY;

    const timer = setTimeout(() => {
      this.initiateDrag(kind, id, fromFolderId, e);
    }, this.dragDelay);

    const moveHandler = (e: PointerEvent) => {
      const deltaX = Math.abs(e.clientX - this.startX);
      const deltaY = Math.abs(e.clientY - this.startY);
      if (deltaX > this.dragThreshold || deltaY > this.dragThreshold) {
        clearTimeout(timer);
        document.removeEventListener("pointermove", moveHandler);
        document.removeEventListener("pointerup", upHandler);
        this.initiateDrag(kind, id, fromFolderId, e);
      }
    };

    const upHandler = () => {
      clearTimeout(timer);
      document.removeEventListener("pointermove", moveHandler);
      document.removeEventListener("pointerup", upHandler);
    };

    document.addEventListener("pointermove", moveHandler);
    document.addEventListener("pointerup", upHandler);
  }

  private initiateDrag(kind: "chat"|"folder", id: ID, fromFolderId: ID | undefined, e: PointerEvent) {
    this.dragState = { kind, id, fromFolderId };
    this.callbacks.onDragStart(kind, id, fromFolderId);

    // Create ghost element
    this.createGhost(e.target as HTMLElement);

    // Add global listeners
    document.addEventListener("pointermove", this.handlePointerMove);
    document.addEventListener("pointerup", this.handlePointerUp);
    document.addEventListener("keydown", this.handleKeyDown);

    // Prevent text selection
    document.body.style.userSelect = "none";
  }

  private createGhost(target: HTMLElement) {
    const rect = target.getBoundingClientRect();
    this.ghostElement = target.cloneNode(true) as HTMLElement;
    this.ghostElement.style.position = "fixed";
    this.ghostElement.style.pointerEvents = "none";
    this.ghostElement.style.zIndex = "9999";
    this.ghostElement.style.opacity = "0.9";
    this.ghostElement.style.transform = "rotate(-2deg)";
    this.ghostElement.style.width = `${rect.width}px`;
    this.ghostElement.style.height = `${rect.height}px`;
    this.ghostElement.style.left = `${rect.left}px`;
    this.ghostElement.style.top = `${rect.top}px`;
    document.body.appendChild(this.ghostElement);
  }

  private handlePointerMove = (e: PointerEvent) => {
    if (!this.dragState || !this.ghostElement) return;

    // Update ghost position
    this.ghostElement.style.left = `${e.clientX - this.ghostElement.offsetWidth / 2}px`;
    this.ghostElement.style.top = `${e.clientY - this.ghostElement.offsetHeight / 2}px`;

    // Find drop target
    const target = this.findDropTarget(e.clientX, e.clientY);
    this.callbacks.onDragMove(target);

    // Handle auto-expand
    this.handleAutoExpand(target);
  };

  private findDropTarget(clientX: number, clientY: number): DragState["over"] {
    const elements = document.elementsFromPoint(clientX, clientY);
    const row = elements.find(el => el.classList.contains("row"));

    if (!row) {
      // Check if over root area
      const tree = document.querySelector(".tree");
      if (tree && this.isPointInRect(clientX, clientY, tree.getBoundingClientRect())) {
        return { type: "root" };
      }
      return undefined;
    }

    const rect = row.getBoundingClientRect();
    const relativeY = clientY - rect.top;
    const height = rect.height;

    // Upper 30% = before, lower 30% = after, middle = on
    if (relativeY < height * 0.3) {
      return { type: "between", id: row.getAttribute("data-id")!, folderId: row.getAttribute("data-folder-id") || undefined };
    } else if (relativeY > height * 0.7) {
      const nextRow = row.nextElementSibling;
      if (nextRow?.classList.contains("row")) {
        return { type: "between", id: nextRow.getAttribute("data-id")!, folderId: nextRow.getAttribute("data-folder-id") || undefined };
      } else {
        return { type: "root" };
      }
    } else {
      const kind = row.classList.contains("folder") ? "folder" : "chat";
      return { type: kind, id: row.getAttribute("data-id")! };
    }
  }

  private isPointInRect(x: number, y: number, rect: DOMRect): boolean {
    return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
  }

  private handleAutoExpand(target: DragState["over"]) {
    if (this.autoExpandTimer) {
      clearTimeout(this.autoExpandTimer);
      this.autoExpandTimer = null;
    }

    if (target?.type === "folder") {
      this.autoExpandTimer = window.setTimeout(() => {
        this.callbacks.onAutoExpand(target.id!);
      }, 500);
    }
  }

  private handlePointerUp = (e: PointerEvent) => {
    if (!this.dragState) return;

    const target = this.findDropTarget(e.clientX, e.clientY);
    this.callbacks.onDragEnd(target);

    this.cleanup();
  };

  private handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      this.callbacks.onDragEnd(undefined);
      this.cleanup();
    }
  };

  private cleanup() {
    if (this.ghostElement) {
      document.body.removeChild(this.ghostElement);
      this.ghostElement = null;
    }

    if (this.autoExpandTimer) {
      clearTimeout(this.autoExpandTimer);
      this.autoExpandTimer = null;
    }

    document.removeEventListener("pointermove", this.handlePointerMove);
    document.removeEventListener("pointerup", this.handlePointerUp);
    document.removeEventListener("keydown", this.handleKeyDown);
    document.body.style.userSelect = "";

    this.dragState = null;
  }
}