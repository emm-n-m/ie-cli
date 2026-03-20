// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.mos;

import java.awt.Point;
import java.util.Objects;

/** Storage for PVRZ-based MOS block information. */
class MosEntry {
  public final int page;
  public final int width;
  public final int height;
  public final Point srcLocation;
  public final Point dstLocation;

  public MosEntry(int page, Point srcLocation, int width, int height, Point dstLocation) {
    this.page = page;
    this.srcLocation = srcLocation;
    this.width = width;
    this.height = height;
    this.dstLocation = dstLocation;
  }

  @Override
  public int hashCode() {
    return Objects.hash(dstLocation, height, page, srcLocation, width);
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
    MosEntry other = (MosEntry)obj;
    return Objects.equals(dstLocation, other.dstLocation) && height == other.height && page == other.page
        && Objects.equals(srcLocation, other.srcLocation) && width == other.width;
  }

  @Override
  public String toString() {
    StringBuilder builder = new StringBuilder();
    builder.append("MosEntry [page=").append(page).append(", width=").append(width).append(", height=").append(height)
        .append(", srcLocation=").append(srcLocation).append(", dstLocation=").append(dstLocation).append("]");
    return builder.toString();
  }
}