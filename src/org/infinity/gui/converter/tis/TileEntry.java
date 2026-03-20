// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.tis;

import java.util.Comparator;
import java.util.Objects;

/** Storage for PVRZ-based TIS tile information. */
public class TileEntry {
  public int tileIndex;
  public int page;
  public int x;
  public int y;

  public static final Comparator<TileEntry> COMPARE_BY_INDEX = Comparator.comparingInt(te -> te.tileIndex);

  public TileEntry(int index, int page, int x, int y) {
    this.tileIndex = index;
    this.page = page;
    this.x = x;
    this.y = y;
  }

  public TileEntry(TileEntry tileEntry) {
    Objects.requireNonNull(tileEntry);
    this.tileIndex = tileEntry.tileIndex;
    this.page = tileEntry.page;
    this.x = tileEntry.x;
    this.y = tileEntry.y;
  }

  @Override
  public int hashCode() {
    return Objects.hash(page, tileIndex, x, y);
  }

  @Override
  public boolean equals(Object obj) {
    if (this == obj) {
      return true;
    }
    if (obj == null) {
      return false;
    }
    if (getClass() != obj.getClass()) {
      return false;
    }
    TileEntry other = (TileEntry)obj;
    return page == other.page && tileIndex == other.tileIndex && x == other.x && y == other.y;
  }

  @Override
  public String toString() {
    return "TileEntry [tileIndex=" + tileIndex + ", page=" + page + ", x=" + x + ", y=" + y + "]";
  }
}