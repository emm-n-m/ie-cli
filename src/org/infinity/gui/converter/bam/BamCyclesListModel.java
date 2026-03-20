// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import javax.swing.AbstractListModel;

import org.infinity.resource.graphics.PseudoBamDecoder;
import org.infinity.resource.graphics.PseudoBamDecoder.PseudoBamCycleEntry;

/** Manages the cycles aspect of BAM resources. */
class BamCyclesListModel extends AbstractListModel<PseudoBamCycleEntry> {
  private final ConvertToBam converter;
  private final PseudoBamDecoder decoder;
  private final PseudoBamDecoder.PseudoBamControl control;

  public BamCyclesListModel(ConvertToBam converter) {
    if (converter == null) {
      throw new NullPointerException();
    }
    this.converter = converter;
    this.decoder = getConverter().getBamDecoder(ConvertToBam.BAM_ORIGINAL);
    this.control = getDecoder().createControl();
  }

  /** Returns the parent converter object. */
  public ConvertToBam getConverter() {
    return converter;
  }

  /** Returns the associated BamDecoder object. */
  public PseudoBamDecoder getDecoder() {
    return decoder;
  }

  /** Returns the associated BamDecoder control. */
  public PseudoBamDecoder.PseudoBamControl getControl() {
    return control;
  }

  // /** Adds a new empty cycle to the cycles list. */
  // public void add()
  // {
  // insert(getControl().cycleCount(), new int[0]);
  // }

  // /** Adds a new cycle with the specified frame index to the cycles list. */
  // public void add(int index)
  // {
  // insert(getControl().cycleCount(), new int[]{index});
  // }

  /** Adds a new cycle with the specified frame indices to the cycles list. */
  public void add(int[] indices) {
    insert(getControl().cycleCount(), indices);
  }

  // /** Insert an empty cycle at the specified cycle position. */
  // public void insert(int pos)
  // {
  // insert(pos, new int[0]);
  // }

  // /** Inserts a cycle with one frame index at the specified cycle position. */
  // public void insert(int pos, int index)
  // {
  // insert(pos, new int[]{index});
  // }

  /** Inserts a cycle with the specified frame indices at the specified cycle position. */
  public void insert(int pos, int[] indices) {
    if (pos >= 0 && pos <= getControl().cycleCount() && indices != null) {
      getControl().cycleInsert(pos, indices);
      fireIntervalAdded(this, pos, pos);
    }
  }

  // /** Removes one cycle at the specified cycle position. */
  // public void remove(int pos)
  // {
  // remove(pos, 1);
  // }

  /** Removes a number of cycles at the specified cycle position. */
  public void remove(int pos, int count) {
    if (pos >= 0 && pos < getControl().cycleCount() && count > 0) {
      if (pos + count > getControl().cycleCount()) {
        count = getControl().cycleCount() - pos;
      }
      getControl().cycleRemove(pos, count);
      if (count > 0) {
        fireIntervalRemoved(this, pos, pos + count - 1);
      }
    }
  }

  /** Removes all cycles from the cycles list. */
  public void clear() {
    int count = getControl().cycleCount();
    getControl().cycleClear();
    if (count > 0) {
      fireIntervalRemoved(this, 0, count - 1);
    }
  }

  /**
   * Moves the specified cycle entry within the cycles list by {@code offset}.
   *
   * @param cycleIdx The index of the cycle.
   * @param offset   The number of positions to move.
   */
  public void move(int cycleIdx, int offset) {
    if (cycleIdx >= 0 && cycleIdx < getControl().cycleCount()) {
      int pos1 = cycleIdx, pos2 = cycleIdx;
      int retVal = getControl().cycleMove(cycleIdx, offset);
      if (retVal >= 0) {
        if (retVal > pos1) {
          pos2 = retVal;
        } else {
          pos1 = retVal;
        }
      }
      if (pos2 > pos1) {
        fireContentsChanged(this, pos1, pos2);
      }
    }
  }

  /** Fires a change event for the cycle at the specified index. */
  public void contentChanged(int index) {
    contentsChanged(index, index);
  }

  /** Fires a change event for the cycle range defined by the specified indices. */
  public void contentsChanged(int index0, int index1) {
    if (index0 >= 0 && index0 < getControl().cycleCount() && index1 >= 0 && index1 < getControl().cycleCount()) {
      if (index0 > index1) {
        int tmp = index0;
        index0 = index1;
        index1 = tmp;
      }
      fireContentsChanged(this, index0, index1);
    }
  }

  /** Returns {@code true} if, and only if {@code getControl().cycleCount()} is 0. */
  public boolean isEmpty() {
    return getControl().isEmpty();
  }

  @Override
  public int getSize() {
    return getControl().cycleCount();
  }

  @Override
  public PseudoBamDecoder.PseudoBamCycleEntry getElementAt(int index) {
    if (index >= 0 && index < getControl().cycleCount()) {
      return getControl().getCycleInfo(index);
    } else {
      return null;
    }
  }
}