// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.image.BufferedImage;
import java.awt.image.DataBuffer;
import java.awt.image.DataBufferByte;
import java.awt.image.DataBufferInt;
import java.awt.image.IndexColorModel;

import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JSlider;
import javax.swing.JSpinner;
import javax.swing.SpinnerNumberModel;
import javax.swing.SwingConstants;
import javax.swing.event.ChangeEvent;
import javax.swing.event.ChangeListener;

import org.infinity.gui.ButtonPopupWindow;
import org.infinity.gui.ViewerUtil;
import org.infinity.icon.Icons;
import org.infinity.resource.graphics.ColorConvert;
import org.infinity.resource.graphics.PseudoBamDecoder.PseudoBamFrameEntry;
import org.infinity.util.Misc;
import org.infinity.util.tuples.Triple;

/**
 * ColorFilter: adjust hue, saturation and lightness.
 */
public class BamFilterColorHSL extends BamFilterBaseColor implements ChangeListener, ActionListener {
  private static final String FILTER_NAME = "Hue/Saturation/Lightness";
  private static final String FILTER_DESC = "This filter provides controls for adjusting hue, "
      + "saturation and lightness";

  private JSlider sliderHue;
  private JSlider sliderSaturation;
  private JSlider sliderLightness;
  private JSpinner spinnerHue;
  private JSpinner spinnerSaturation;
  private JSpinner spinnerLightness;
  private ButtonPopupWindow bpwExclude;
  private BamFilterBaseColor.ExcludeColorsPanel pExcludeColors;

  public static String getFilterName() {
    return FILTER_NAME;
  }

  public static String getFilterDesc() {
    return FILTER_DESC;
  }

  public BamFilterColorHSL(ConvertToBam parent) {
    super(parent, FILTER_NAME, FILTER_DESC);
  }

  @Override
  public BufferedImage process(BufferedImage frame) throws Exception {
    return applyEffect(frame);
  }

  @Override
  public PseudoBamFrameEntry updatePreview(int frameIndex, PseudoBamFrameEntry entry) {
    if (entry != null) {
      entry.setFrame(applyEffect(entry.getFrame()));
    }
    return entry;
  }

  @Override
  public void updateControls() {
    bpwExclude.setEnabled(getConverter().isBamV1Selected());
  }

  @Override
  public String getConfiguration() {
    return String.valueOf(sliderHue.getValue()) + ';' +
        sliderSaturation.getValue() + ';' +
        sliderLightness.getValue() + ';' +
        encodeColorList(pExcludeColors.getSelectedIndices());
  }

  @Override
  public boolean setConfiguration(String config) {
    if (config != null) {
      config = config.trim();
      if (!config.isEmpty()) {
        String[] params = config.trim().split(";");
        int hValue = Integer.MIN_VALUE;
        int sValue = Integer.MIN_VALUE;
        int lValue = Integer.MIN_VALUE;
        int[] indices = null;

        // parsing configuration data
        if (params.length > 0) { // set hue value
          hValue = decodeNumber(params[0], sliderHue.getMinimum(), sliderHue.getMaximum(), Integer.MIN_VALUE);
          if (hValue == Integer.MIN_VALUE) {
            return false;
          }
        }
        if (params.length > 1) { // set saturation value
          sValue = decodeNumber(params[1], sliderSaturation.getMinimum(), sliderSaturation.getMaximum(),
              Integer.MIN_VALUE);
          if (sValue == Integer.MIN_VALUE) {
            return false;
          }
        }
        if (params.length > 2) { // set lightness value
          lValue = decodeNumber(params[2], sliderLightness.getMinimum(), sliderLightness.getMaximum(),
              Integer.MIN_VALUE);
          if (lValue == Integer.MIN_VALUE) {
            return false;
          }
        }
        if (params.length > 3) {
          indices = decodeColorList(params[3]);
          if (indices == null) {
            return false;
          }
        }

        // applying configuration data
        if (hValue != Integer.MIN_VALUE) {
          sliderHue.setValue(hValue);
        }
        if (sValue != Integer.MIN_VALUE) {
          sliderSaturation.setValue(sValue);
        }
        if (lValue != Integer.MIN_VALUE) {
          sliderLightness.setValue(lValue);
        }
        if (indices != null) {
          pExcludeColors.setSelectedIndices(indices);
        }
      }
      return true;
    }
    return false;
  }

  @Override
  protected JPanel loadControls() {
    GridBagConstraints c = new GridBagConstraints();

    JLabel l1 = new JLabel("Exclude colors:");
    pExcludeColors = new BamFilterBaseColor.ExcludeColorsPanel(
        getConverter().getPaletteDialog().getPalette(getConverter().getPaletteDialog().getPaletteType()));
    pExcludeColors.addChangeListener(this);
    bpwExclude = new ButtonPopupWindow("Palette", Icons.ICON_ARROW_DOWN_15.getIcon(), pExcludeColors);
    bpwExclude.setIconTextGap(8);
    bpwExclude.addActionListener(this);
    bpwExclude.setEnabled(getConverter().isBamV1Selected());
    JPanel pExclude = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    pExclude.add(l1, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    pExclude.add(bpwExclude, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 4, 0, 0), 0, 0);
    pExclude.add(new JPanel(), c);

    JLabel lh = new JLabel("Hue:");
    JLabel ls = new JLabel("Saturation:");
    JLabel ll = new JLabel("Lightness:");
    sliderHue = new JSlider(SwingConstants.HORIZONTAL, -180, 180, 0);
    sliderHue.addChangeListener(this);
    sliderSaturation = new JSlider(SwingConstants.HORIZONTAL, -100, 100, 0);
    sliderSaturation.addChangeListener(this);
    sliderLightness = new JSlider(SwingConstants.HORIZONTAL, -100, 100, 0);
    sliderLightness.addChangeListener(this);
    spinnerHue = new JSpinner(
        new SpinnerNumberModel(sliderHue.getValue(), sliderHue.getMinimum(), sliderHue.getMaximum(), 1));
    spinnerHue.addChangeListener(this);
    spinnerSaturation = new JSpinner(new SpinnerNumberModel(sliderSaturation.getValue(), sliderSaturation.getMinimum(),
        sliderSaturation.getMaximum(), 1));
    spinnerSaturation.addChangeListener(this);
    spinnerLightness = new JSpinner(new SpinnerNumberModel(sliderLightness.getValue(), sliderLightness.getMinimum(),
        sliderLightness.getMaximum(), 1));
    spinnerLightness.addChangeListener(this);

    JPanel p = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    p.add(lh, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 4, 0, 0), 0, 0);
    p.add(sliderHue, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 4, 0, 0), 0, 0);
    p.add(spinnerHue, c);

    ViewerUtil.setGBC(c, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(4, 0, 0, 0), 0, 0);
    p.add(ls, c);
    ViewerUtil.setGBC(c, 1, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 0, 0), 0, 0);
    p.add(sliderSaturation, c);
    ViewerUtil.setGBC(c, 2, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 0, 0), 0, 0);
    p.add(spinnerSaturation, c);

    ViewerUtil.setGBC(c, 0, 2, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(4, 0, 0, 0), 0, 0);
    p.add(ll, c);
    ViewerUtil.setGBC(c, 1, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 0, 0), 0, 0);
    p.add(sliderLightness, c);
    ViewerUtil.setGBC(c, 2, 2, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 0, 0), 0, 0);
    p.add(spinnerLightness, c);
    ViewerUtil.setGBC(c, 0, 3, 3, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(8, 0, 0, 0), 0, 0);
    p.add(pExclude, c);

    JPanel panel = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panel.add(p, c);

    return panel;
  }

  // --------------------- Begin Interface ChangeListener ---------------------

  @Override
  public void stateChanged(ChangeEvent event) {
    if (event.getSource() == pExcludeColors) {
      fireChangeListener();
    } else if (event.getSource() == sliderHue) {
      spinnerHue.setValue(sliderHue.getValue());
      if (!sliderHue.getModel().getValueIsAdjusting()) {
        fireChangeListener();
      }
    } else if (event.getSource() == sliderSaturation) {
      spinnerSaturation.setValue(sliderSaturation.getValue());
      if (!sliderSaturation.getModel().getValueIsAdjusting()) {
        fireChangeListener();
      }
    } else if (event.getSource() == sliderLightness) {
      spinnerLightness.setValue(sliderLightness.getValue());
      if (!sliderLightness.getModel().getValueIsAdjusting()) {
        fireChangeListener();
      }
    } else if (event.getSource() == spinnerHue) {
      sliderHue.setValue(((Integer) spinnerHue.getValue()));
    } else if (event.getSource() == spinnerSaturation) {
      sliderSaturation.setValue(((Integer) spinnerSaturation.getValue()));
    } else if (event.getSource() == spinnerLightness) {
      sliderLightness.setValue(((Integer) spinnerLightness.getValue()));
    }
  }

  // --------------------- End Interface ChangeListener ---------------------

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent event) {
    if (event.getSource() == bpwExclude) {
      pExcludeColors.updatePalette(
          getConverter().getPaletteDialog().getPalette(getConverter().getPaletteDialog().getPaletteType()));
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  private BufferedImage applyEffect(BufferedImage srcImage) {
    if (srcImage != null) {
      int[] buffer;
      IndexColorModel cm = null;
      boolean isPremultiplied = false;
      if (srcImage.getType() == BufferedImage.TYPE_BYTE_INDEXED) {
        // paletted image
        cm = (IndexColorModel) srcImage.getColorModel();
        buffer = new int[1 << cm.getPixelSize()];
        cm.getRGBs(buffer);
        isPremultiplied = cm.isAlphaPremultiplied();
        // applying proper alpha
        if (!cm.hasAlpha()) {
          final int Green = 0x0000ff00;
          boolean greenFound = false;
          for (int i = 0; i < buffer.length; i++) {
            if (!greenFound && buffer[i] == Green) {
              greenFound = true;
              buffer[i] &= 0x00ffffff;
            } else {
              buffer[i] |= 0xff000000;
            }
          }
        }
      } else if (srcImage.getRaster().getDataBuffer().getDataType() == DataBuffer.TYPE_INT) {
        // truecolor image
        buffer = ((DataBufferInt) srcImage.getRaster().getDataBuffer()).getData();
        isPremultiplied = srcImage.isAlphaPremultiplied();
      } else {
        buffer = new int[0];
      }

      // hue in range [-180, 180]
      float hue = ((Integer) spinnerHue.getValue()).floatValue() / 360.0f;
      // saturation in range [-100, 100]
      float saturation = ((Integer) spinnerSaturation.getValue()).floatValue() / 100.0f;
      // lightness in range [-100, 100]
      float lightness = ((Integer) spinnerLightness.getValue()).floatValue() / 100.0f;

      for (int i = 0; i < buffer.length; i++) {
        if ((cm == null || !pExcludeColors.isSelectedIndex(i)) && (buffer[i] & 0xff000000) != 0) {
          // convert RGB -> HSL
          final Triple<Double, Double, Double> hsl = ColorConvert.convertRGBtoHSL(buffer[i], isPremultiplied);

          // applying adjustments
          double h = (hsl.getValue0() + hue) % 1.0;
          if (h < 0.0) {
            h += 1.0;
          }
          double s = Misc.clamp(hsl.getValue1() + saturation, 0.0, 1.0);
          double l = Misc.clamp(hsl.getValue2() + lightness, 0.0, 1.0);

          // converting HSL -> RGB
          buffer[i] = ColorConvert.convertHSLtoRGB(h, s, l, buffer[i] >>> 24, isPremultiplied);
        }
      }

      if (cm != null) {
        // recreating paletted image
        IndexColorModel cm2 = new IndexColorModel(cm.getPixelSize(), buffer.length, buffer, 0, cm.hasAlpha(),
            cm.getTransparentPixel(), DataBuffer.TYPE_BYTE);
        int width = srcImage.getWidth();
        int height = srcImage.getHeight();
        BufferedImage dstImage = new BufferedImage(width, height, BufferedImage.TYPE_BYTE_INDEXED, cm2);
        byte[] srcPixels = ((DataBufferByte) srcImage.getRaster().getDataBuffer()).getData();
        byte[] dstPixels = ((DataBufferByte) dstImage.getRaster().getDataBuffer()).getData();
        System.arraycopy(srcPixels, 0, dstPixels, 0, srcPixels.length);
        srcImage = dstImage;
      }
    }

    return srcImage;
  }
}
