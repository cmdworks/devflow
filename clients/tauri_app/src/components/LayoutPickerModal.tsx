import React, { useEffect, useRef } from "react";
import {
  Square,
  Columns,
  Rows,
  Grid,
  Check,
  X,
  Sparkles,
} from "lucide-react";
import type { LayoutMode } from "./TerminalSecondaryBar";

interface LayoutPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  layoutMode: LayoutMode;
  onChangeLayout: (mode: LayoutMode) => void;
}

interface LayoutOption {
  id: LayoutMode;
  title: string;
  subtitle: string;
  diagram: React.ReactNode;
  icon: React.ReactNode;
}

export const LayoutPickerModal: React.FC<LayoutPickerModalProps> = ({
  isOpen,
  onClose,
  layoutMode,
  onChangeLayout,
}) => {
  const modalRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (modalRef.current && !modalRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      document.addEventListener("keydown", handleKeyDown);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const options: LayoutOption[] = [
    {
      id: "tabs",
      title: "Single Focus (Tabs)",
      subtitle: "100% viewport focused on one active target pane",
      icon: <Square size={16} />,
      diagram: (
        <div className="layout-preview-box">
          <div className="preview-pane full active">100%</div>
        </div>
      ),
    },
    {
      id: "split-h",
      title: "Side by Side (Horizontal)",
      subtitle: "Two vertical columns for parallel target diff & log monitoring",
      icon: <Columns size={16} />,
      diagram: (
        <div className="layout-preview-box horizontal">
          <div className="preview-pane half active">50%</div>
          <div className="preview-pane half">50%</div>
        </div>
      ),
    },
    {
      id: "split-v",
      title: "Stacked (Vertical)",
      subtitle: "Upper and lower split rows with independent scrollback",
      icon: <Rows size={16} />,
      diagram: (
        <div className="layout-preview-box vertical">
          <div className="preview-pane half active">Top</div>
          <div className="preview-pane half">Bottom</div>
        </div>
      ),
    },
    {
      id: "grid",
      title: "2×2 Quad Grid",
      subtitle: "Four quadrants for comprehensive multi-target orchestrations",
      icon: <Grid size={16} />,
      diagram: (
        <div className="layout-preview-box grid">
          <div className="preview-pane quad active">1</div>
          <div className="preview-pane quad">2</div>
          <div className="preview-pane quad">3</div>
          <div className="preview-pane quad">4</div>
        </div>
      ),
    },
  ];

  return (
    <div className="layout-picker-anchor" ref={modalRef}>
      <div className="layout-picker-card">
        <div className="layout-picker-header">
          <div className="layout-picker-title-group">
            <Sparkles size={13} color="#06b6d4" />
            <span className="layout-picker-title">Customize Terminal Layout</span>
          </div>
          <button className="btn-picker-close" onClick={onClose} title="Close layout picker">
            <X size={13} />
          </button>
        </div>

        <div className="layout-options-grid">
          {options.map((opt) => {
            const isSelected = layoutMode === opt.id;
            return (
              <div
                key={opt.id}
                className={`layout-option-card ${isSelected ? "selected" : ""}`}
                onClick={() => {
                  onChangeLayout(opt.id);
                  onClose();
                }}
              >
                <div className="layout-option-top">
                  <div className="layout-option-icon">{opt.icon}</div>
                  <div className="layout-option-info">
                    <div className="layout-option-name">
                      <span>{opt.title}</span>
                      {isSelected && <Check size={13} color="#06b6d4" />}
                    </div>
                    <div className="layout-option-desc">{opt.subtitle}</div>
                  </div>
                </div>

                <div className="layout-option-preview">
                  {opt.diagram}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
