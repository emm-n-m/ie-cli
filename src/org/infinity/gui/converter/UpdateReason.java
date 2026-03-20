// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.io.Serializable;
import java.util.Objects;

/**
 * Used for constants that provide information about panel update reasons.
 *
 * @see {@link PanelUpdateEvent#getReason()}
 */
public final class UpdateReason implements Serializable, Comparable<Integer> {
  private static final long serialVersionUID = 1L | (long)UpdateReason.class.hashCode() << 32;

  private final int id;

  /**
   * Initializes a new {@code UpdateReason}.
   *
   * @param id Numeric identifier that should be unique in the panel context.
   */
  public UpdateReason(int id) {
    this.id = id;
  }

  /** Returns the numeric identifier associated with this object. */
  public int getId() {
    return id;
  }

  @Override
  public int hashCode() {
    return Objects.hash(id);
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
    UpdateReason other = (UpdateReason)obj;
    return id == other.id;
  }

  @Override
  public String toString() {
    return Integer.toString(id);
  }

  @Override
  public int compareTo(Integer o) {
    if (o != null) {
      return (id > o) ? 1 : (id < o) ? -1 : 0;
    } else {
      return (id > 0) ? 1 : (id < 0) ? -1 : 0;
    }
  }
}