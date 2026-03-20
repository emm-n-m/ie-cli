// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.Point;
import java.awt.image.BufferedImage;

import javax.swing.AbstractListModel;

import org.infinity.resource.graphics.BamDecoder;
import org.infinity.resource.graphics.ColorConvert;
import org.infinity.resource.graphics.PseudoBamDecoder;
import org.infinity.resource.graphics.PseudoBamDecoder.PseudoBamControl;
import org.infinity.resource.graphics.PseudoBamDecoder.PseudoBamFrameEntry;

/** Manages the frames aspect of BAM resources. */
class BamFramesListModel extends AbstractListModel<PseudoBamFrameEntry> {
  private final ConvertToBam converter;
  private final PseudoBamDecoder decoder;

  public BamFramesListModel(ConvertToBam converter) {
    if (converter == null) {
      throw new NullPointerException();
    }
    this.converter = converter;
    this.decoder = this.converter.getBamDecoder(ConvertToBam.BAM_ORIGINAL);
  }

  /** Returns the parent converter object. */
  public ConvertToBam getConverter() {
    return converter;
  }

  /** Returns the associated BamDecoder object. */
  public PseudoBamDecoder getDecoder() {
    return decoder;
  }

  // /** Adds a new frame to the global frames list. */
  // public void add(BufferedImage image, Point center)
  // {
  // insert(getDecoder().frameCount(), new BufferedImage[]{image}, new Point[]{center});
  // }

  // /** Adds an array of images to the global frames list. */
  // public void add(BufferedImage[] images, Point[] centers)
  // {
  // insert(getDecoder().frameCount(), images, centers);
  // }

  /** Inserts the image into the global frames list. */
  public void insert(int pos, BufferedImage image, Point center) {
    insert(pos, image, center, false);
  }

  /** Inserts the image into the global frames list. */
  public void insert(int pos, BufferedImage image, Point center, boolean forceTransparentGreen) {
    insert(pos, new BufferedImage[] { image }, new Point[] { center }, forceTransparentGreen);
  }

  /** Inserts the array of images into the global frames list. */
  public void insert(int pos, BufferedImage[] images, Point[] centers, boolean forceTransparentGreen) {
    if (images != null && pos >= 0 && pos <= getDecoder().frameCount()) {
      int count = 0;
      PseudoBamControl control = getDecoder().createControl();
      control.setMode(BamDecoder.BamControl.Mode.INDIVIDUAL);
      for (int i = 0; i < images.length; i++) {
        if (images[i] != null) {
          // adding frame to global list
          Point center = (centers.length > i && centers[i] != null) ? centers[i] : null;
          final int frameIdx = pos + i;
          getDecoder().frameInsert(frameIdx, images[i], center);
          getDecoder().getFrameInfo(frameIdx).setOption(PseudoBamDecoder.OPTION_BOOL_TRANSPARENTGREENFORCED,
            forceTransparentGreen);
          // registering colors values in global HashMap
          BufferedImage image = ColorConvert.toBufferedImage(images[i], true, false);
          PseudoBamDecoder.registerColors(getConverter().getPaletteDialog().getColorMap(), image, forceTransparentGreen);
          getConverter().getPaletteDialog().setPaletteModified();
          count++;
        }
      }
      if (count > 0) {
        fireIntervalAdded(this, pos, pos + count - 1);
      }
    }
  }

  /**
   * Removes a single entry from the global frames list.
   */
  public void remove(int pos) {
    remove(pos, 1);
  }

  /**
   * Removes a number of entries from the global frames list.
   */
  public void remove(int pos, int count) {
    if (count > 0) {
      if (pos < 0) {
        pos = 0;
      }
      if (pos >= getDecoder().frameCount()) {
        pos = getDecoder().frameCount() - 1;
      }
      if (pos + count > getDecoder().frameCount()) {
        count = getDecoder().frameCount() - pos;
      }
      PseudoBamControl control = getDecoder().createControl();
      control.setMode(BamDecoder.BamControl.Mode.INDIVIDUAL);
      // unregistering color values in global color map
      for (int i = 0; i < count; i++) {
        final int frameIdx = pos + i;
        BufferedImage image = ColorConvert.toBufferedImage(getDecoder().frameGet(control, frameIdx), true, false);
        PseudoBamDecoder.unregisterColors(getConverter().getPaletteDialog().getColorMap(), image, (boolean)getDecoder()
          .getFrameInfo(frameIdx).getOption(PseudoBamDecoder.OPTION_BOOL_TRANSPARENTGREENFORCED));
        getConverter().getPaletteDialog().setPaletteModified();
      }
      getDecoder().frameRemove(pos, count);
      fireIntervalRemoved(this, pos, pos + count - 1);
    }
  }

  /**
   * Removes all entries from the global frame list.
   */
  public void clear() {
    int count;
    count = getDecoder().frameCount();
    PseudoBamControl control = getDecoder().createControl();
    control.setMode(BamDecoder.BamControl.Mode.INDIVIDUAL);
    // clearing global color map
    getConverter().getPaletteDialog().getColorMap().clear();
    getConverter().getPaletteDialog().setPaletteModified();
    getDecoder().frameClear();
    if (count > 0) {
      fireIntervalRemoved(this, 0, count - 1);
    }
  }

  /**
   * Moves the specified entry within the list by {@code offset}.
   *
   * @param index  The index of the frame.
   * @param offset The number of positions to move.
   */
  public void move(int index, int offset) {
    int retVal, pos1 = index, pos2 = index;

    // moving positions
    retVal = getDecoder().frameMove(index, offset);

    // preparing interval
    if (retVal >= 0) {
      if (retVal > pos1) {
        pos2 = retVal;
      } else {
        pos1 = retVal;
      }
    }

    // notifying listeners
    if (pos2 > pos1) {
      fireContentsChanged(this, pos1, pos2);
    }
  }

  /** Returns {@code true} if, and only if {@code getDecoder().getFrameCount()} is 0. */
  public boolean isEmpty() {
    return getDecoder().isEmpty();
  }

  @Override
  public int getSize() {
    return getDecoder().frameCount();
  }

  @Override
  public PseudoBamFrameEntry getElementAt(int index) {
    if (index >= 0 && index < getDecoder().frameCount()) {
      return getDecoder().getFrameInfo(index);
    } else {
      return null;
    }
  }
}