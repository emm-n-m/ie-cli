// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui;

import java.awt.AlphaComposite;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.Transparency;
import java.awt.event.FocusEvent;
import java.awt.event.FocusListener;
import java.awt.event.KeyEvent;
import java.awt.event.KeyListener;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;
import java.awt.event.MouseMotionListener;
import java.awt.image.BufferedImage;
import java.awt.image.DataBuffer;
import java.awt.image.DataBufferInt;

import javax.swing.BorderFactory;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JTextField;

import org.infinity.resource.graphics.ColorConvert;
import org.infinity.util.Misc;

/**
 * Implements controls to choose an arbitrary color value, either visually, as RGB(A) value, or HSB value.
 */
public class ColorChooser extends JPanel {
  /** Supported color formats. */
  public enum ColorFormat {
    /** Byte order: {alpha, red, green, blue} */
    ARGB(8, 16, 24, 0),
    /** Byte order: {red, green, blue, alpha} */
    RGBA(0, 8, 16, 24),
    /** Byte order: {blue, green red, alpha} */
    BGRA(16, 8, 0, 24),
    /** Byte order: {alpha, blue, green, red} */
    ABGR(24, 16, 8, 0),
    ;

    private final int shiftRed;
    private final int shiftGreen;
    private final int shiftBlue;
    private final int shiftAlpha;

    ColorFormat(int shiftRed, int shiftGreen, int shiftBlue, int shiftAlpha) {
      this.shiftRed = shiftRed;
      this.shiftGreen = shiftGreen;
      this.shiftBlue = shiftBlue;
      this.shiftAlpha = shiftAlpha;
    }

    /** Returns the combined color in the specified format. */
    public int getRGBA(int red, int green, int blue, int alpha) {
      return ((red & 0xff) << shiftRed) |
          ((green & 0xff) << shiftGreen) |
          ((blue & 0xff) << shiftBlue) |
          ((alpha & 0xff) << shiftAlpha);
    }

    /** Returns the red color component value. */
    public int getRed(int color) {
      return (color >>> shiftRed) & 0xff;
    }

    /** Returns the green color component value. */
    public int getGreen(int color) {
      return (color >>> shiftGreen) & 0xff;
    }

    /** Returns the blue color component value. */
    public int getBlue(int color) {
      return (color >>> shiftBlue) & 0xff;
    }

    /** Returns the alpha component value. */
    public int getAlpha(int color) {
      return (color >>> shiftAlpha) & 0xff;
    }
  }

  private final Listeners listeners = new Listeners();
  private final ColorFormat format;
  private final boolean alphaEnabled;

  private RenderCanvas rcMainPreview, rcSecondPreview, rcColorPreview;
  private JTextField tfHue, tfSat, tfBri, tfRed, tfGreen, tfBlue, tfAlpha;
  private int tmpHue, tmpSat, tmpBri, tmpRed, tmpGreen, tmpBlue, tmpAlpha;
  private int prevValue;

  /**
   * Helper method that returns a correctly encoded color value based on the specified parameters.
   *
   * @param r            Red color component.
   * @param g            Green color component.
   * @param b            Blue color component.
   * @param a            Alpha component. Ignored if {@code alphaEnabled} is {@code false}.
   * @param format       {@link ColorFormat} enum with the desired color format. {@link ColorFormat#BGRA} is used if
   *                       {@code null} is specified.
   * @param alphaEnabled Specifies whether the alpha value of the color can be specified.
   * @return Encoded color value.
   */
  public static int getEncodedColor(int r, int g, int b, int a, ColorFormat format, boolean alphaEnabled) {
    format = (format == null) ? ColorFormat.BGRA : format;
    a = alphaEnabled ? a : 0;
    return format.getRGBA(r, g, b, a);
  }

  /** Checks and returns either the number fetched from the input field or oldVal on error. */
  private static int validateNumberInput(JTextField tf, int oldVal, int min, int max) {
    if (tf != null) {
      oldVal = Misc.toNumber(tf.getText(), 0);
      oldVal = Misc.clamp(oldVal, min, max);
      tf.setText(Integer.toString(oldVal));
    }
    return oldVal;
  }

  /**
   * Initializes the color chooser panel with default value (ColorFormat: BGRA, color: black, no alpha).
   */
  public ColorChooser() {
    this(null, getEncodedColor(0, 0, 0, 255, null, false), false);
  }

  /**
   * Initializes the color chooser panel.
   *
   * @param format       {@link ColorFormat} enum with the color format. {@link ColorFormat#BGRA} is used if
   *                       {@code null} is specified.
   * @param alphaEnabled Specifies whether the alpha value of the color can be specified.
   */
  public ColorChooser(ColorFormat format, boolean alphaEnabled) {
    this(format, getEncodedColor(0, 0, 0, 255, format, alphaEnabled), alphaEnabled);
  }

  /**
   * Initializes the color chooser panel.
   *
   * @param format       {@link ColorFormat} instance with the color format. {@link ColorFormat#BGRA} is used if
   *                       {@code null} is specified.
   * @param argb         Initial color as an integer value in the specified {@code format}.
   * @param alphaEnabled Specifies whether the alpha value of the color can be specified.
   */
  public ColorChooser(ColorFormat format, int argb, boolean alphaEnabled) {
    super(true);
    this.format = (format != null) ? format : ColorFormat.BGRA;
    this.alphaEnabled = alphaEnabled;
    init(argb);
  }

  /** Returns whether the alpha channel is enabled. */
  public boolean isAlphaEnabled() {
    return alphaEnabled;
  }

  /** Returns the {@link ColorFormat} enum used by this component. */
  public ColorFormat getColorFormat() {
    return format;
  }

  /** Returns the currently selected color value in the format returned by {@link #getColorFormat()}. */
  public int getColor() {
    return getInputRgbaValue();
  }

  /** Returns the color value this panel has been initialized with. */
  public int getInitialColor() {
    return prevValue;
  }

  /**
   * Sets the initial color to a new value.
   *
   * @param argb New initial color value in the format returned by {@link #getColorFormat()}.
   */
  public void resetInitialColor(int argb) {
    prevValue = getEncodedColor(format.getRed(argb), format.getGreen(argb), format.getBlue(argb), format.getAlpha(argb),
        format, alphaEnabled);
    tmpRed = format.getRed(prevValue);
    tmpGreen = format.getGreen(prevValue);
    tmpBlue = format.getBlue(prevValue);
    tmpAlpha = format.getAlpha(prevValue);
    float[] hsb = { 0.0f, 0.0f, 0.0f };
    Color.RGBtoHSB(tmpRed, tmpGreen, tmpBlue, hsb);
    tmpHue = Math.round(hsb[0] * 360.0f);
    tmpSat = Math.round(hsb[1] * 100.0f);
    tmpBri = Math.round(hsb[2] * 100.0f);

    updateColorValues(prevValue);
    updateColor();
  }

  /**
   * Initializes UI controls.
   *
   * @param argb Initial color value
   */
  private void init(int argb) {
    rcMainPreview = new RenderCanvas(); // use H as x and B as y
    rcMainPreview.setSize(Misc.getScaledValue(256), Misc.getScaledValue(128));
    rcMainPreview.setScalingEnabled(true);
    rcMainPreview.setPreferredSize(rcMainPreview.getSize());
    rcMainPreview.setBorder(BorderFactory.createLineBorder(Color.BLACK));
    rcMainPreview.addMouseListener(listeners);
    rcMainPreview.addMouseMotionListener(listeners);
    rcMainPreview
        .setImage(ColorConvert.createCompatibleImage(rcMainPreview.getWidth(), rcMainPreview.getHeight(), false));

    rcSecondPreview = new RenderCanvas(); // use S
    rcSecondPreview.setSize(Misc.getScaledValue(32), Misc.getScaledValue(128));
    rcSecondPreview.setScalingEnabled(true);
    rcSecondPreview.setPreferredSize(rcSecondPreview.getSize());
    rcSecondPreview.setBorder(BorderFactory.createLineBorder(Color.BLACK));
    rcSecondPreview.addMouseListener(listeners);
    rcSecondPreview.addMouseMotionListener(listeners);
    rcSecondPreview
        .setImage(ColorConvert.createCompatibleImage(rcSecondPreview.getWidth(), rcSecondPreview.getHeight(), false));

    final JLabel lPreview = new JLabel("Preview");
    rcColorPreview = new RenderCanvas(); // shows currently defined color
    rcColorPreview.setSize(Misc.getScaledValue(64), Misc.getScaledValue(32));
    rcColorPreview.setScalingEnabled(true);
    rcColorPreview.setPreferredSize(rcColorPreview.getSize());
    rcColorPreview.setBorder(BorderFactory.createLineBorder(Color.BLACK));
    rcColorPreview.setBackground(Color.BLACK);
    rcColorPreview.setImage(ColorConvert.createCompatibleImage(rcColorPreview.getWidth(), rcColorPreview.getHeight(),
        Transparency.TRANSLUCENT));

    final JLabel lHue = new JLabel("H:");
    lHue.setToolTipText("Hue");
    final JLabel lSat = new JLabel("S:");
    lSat.setToolTipText("Saturation");
    final JLabel lBri = new JLabel("B:");
    lBri.setToolTipText("Brightness");

    tfHue = new JTextField(4); // range: [0..359]
    tfHue.addFocusListener(listeners);
    tfHue.addKeyListener(listeners);
    tfSat = new JTextField(4); // range: [0..100]
    tfSat.addFocusListener(listeners);
    tfSat.addKeyListener(listeners);
    tfBri = new JTextField(4); // range: [0..100]
    tfBri.addFocusListener(listeners);
    tfBri.addKeyListener(listeners);

    final JLabel lHue2 = new JLabel("°");
    final JLabel lSat2 = new JLabel("%");
    final JLabel lBri2 = new JLabel("%");

    final JLabel lR = new JLabel("R:");
    lBri.setToolTipText("Red");
    final JLabel lG = new JLabel("G:");
    lG.setToolTipText("Green");
    final JLabel lB = new JLabel("B:");
    lB.setToolTipText("Blue");
    final JLabel lA;
    if (isAlphaEnabled()) {
      lA = new JLabel("A:");
      lA.setToolTipText("Alpha");
    } else {
      lA = null;
    }

    tfRed = new JTextField(4); // range: [0..255]
    tfRed.addFocusListener(listeners);
    tfRed.addKeyListener(listeners);
    tfGreen = new JTextField(4); // range: [0..255]
    tfGreen.addFocusListener(listeners);
    tfGreen.addKeyListener(listeners);
    tfBlue = new JTextField(4); // range: [0..255]
    tfBlue.addFocusListener(listeners);
    tfBlue.addKeyListener(listeners);
    if (isAlphaEnabled()) {
      tfAlpha= new JTextField(4); // range: [0..255]
      tfAlpha.addFocusListener(listeners);
      tfAlpha.addKeyListener(listeners);
    }

    final GridBagConstraints gbc = new GridBagConstraints();

    // Setting up HSB controls
    final JPanel pHSB = new JPanel(new GridBagLayout());

    ViewerUtil.setGBC(gbc, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    pHSB.add(lHue, gbc);
    ViewerUtil.setGBC(gbc, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 4, 0, 0), 0, 0);
    pHSB.add(tfHue, gbc);
    ViewerUtil.setGBC(gbc, 2, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 4, 0, 0), 0, 0);
    pHSB.add(lHue2, gbc);

    ViewerUtil.setGBC(gbc, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 0, 0, 0), 0, 0);
    pHSB.add(lSat, gbc);
    ViewerUtil.setGBC(gbc, 1, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 4, 0, 0), 0, 0);
    pHSB.add(tfSat, gbc);
    ViewerUtil.setGBC(gbc, 2, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 4, 0, 0), 0, 0);
    pHSB.add(lSat2, gbc);

    ViewerUtil.setGBC(gbc, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 0, 0, 0), 0, 0);
    pHSB.add(lBri, gbc);
    ViewerUtil.setGBC(gbc, 1, 2, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 4, 0, 0), 0, 0);
    pHSB.add(tfBri, gbc);
    ViewerUtil.setGBC(gbc, 2, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 4, 0, 0), 0, 0);
    pHSB.add(lBri2, gbc);

    // Setting up RGBA controls
    final JPanel pRGBA = new JPanel(new GridBagLayout());

    ViewerUtil.setGBC(gbc, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    pRGBA.add(lR, gbc);
    ViewerUtil.setGBC(gbc, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 4, 0, 0), 0, 0);
    pRGBA.add(tfRed, gbc);

    ViewerUtil.setGBC(gbc, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 0, 0, 0), 0, 0);
    pRGBA.add(lG, gbc);
    ViewerUtil.setGBC(gbc, 1, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 4, 0, 0), 0, 0);
    pRGBA.add(tfGreen, gbc);

    ViewerUtil.setGBC(gbc, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 0, 0, 0), 0, 0);
    pRGBA.add(lB, gbc);
    ViewerUtil.setGBC(gbc, 1, 2, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(2, 4, 0, 0), 0, 0);
    pRGBA.add(tfBlue, gbc);

    if (isAlphaEnabled()) {
      ViewerUtil.setGBC(gbc, 0, 3, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(2, 0, 0, 0), 0, 0);
      pRGBA.add(lA, gbc);
      ViewerUtil.setGBC(gbc, 1, 3, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(2, 4, 0, 0), 0, 0);
      pRGBA.add(tfAlpha, gbc);
    }

    // Setting up color preview
    final JPanel pPreview = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(gbc, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    pPreview.add(lPreview, gbc);
    ViewerUtil.setGBC(gbc, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    pPreview.add(rcColorPreview, gbc);

    // Setting up main controls
    JPanel pControls = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(gbc, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    pControls.add(pHSB, gbc);
    ViewerUtil.setGBC(gbc, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 16, 0, 0), 0, 0);
    pControls.add(pRGBA, gbc);
    ViewerUtil.setGBC(gbc, 0, 1, 2, 1, 1.0, 1.0, GridBagConstraints.LAST_LINE_START,
        GridBagConstraints.HORIZONTAL, new Insets(0, 0, 0, 0), 0, 0);
    pControls.add(new JPanel(), gbc);
    ViewerUtil.setGBC(gbc, 0, 2, 2, 1, 1.0, 0.0, GridBagConstraints.LAST_LINE_START,
        GridBagConstraints.HORIZONTAL, new Insets(0, 0, 0, 0), 0, 0);
    pControls.add(pPreview, gbc);

    setLayout(new GridBagLayout());
    ViewerUtil.setGBC(gbc, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(8, 8, 8, 0), 0, 0);
    add(rcMainPreview, gbc);
    ViewerUtil.setGBC(gbc, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(8, 8, 8, 0), 0, 0);
    add(rcSecondPreview, gbc);
    ViewerUtil.setGBC(gbc, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(8, 16, 8, 8), 0, 0);
    add(pControls, gbc);

    resetInitialColor(argb);
    updateColorValues(getInitialColor());
  }

  /** Returns RGBA value from HSB parameters: h, s, b in range [0..1] */
  private int getHsbValue(float h, float s, float b) {
    h = Misc.clamp(h, 0.0f, 1.0f);
    s = Misc.clamp(s, 0.0f, 1.0f);
    b = Misc.clamp(b, 0.0f, 1.0f);
    final Color c = new Color(Color.HSBtoRGB(h, s, b));
    return format.getRGBA(c.getRed(), c.getGreen(), c.getBlue(), c.getAlpha());
  }

  /** Returns a color value based on the RGB input fields. */
  private int getInputRgbaValue() {
    return format.getRGBA(getInputRed(), getInputGreen(), getInputBlue(), getInputAlpha());
  }

  /** Returns current red color value. Value in range [0..255] */
  private int getInputRed() {
    int v = Misc.toNumber(tfRed.getText(), 0);
    return Misc.clamp(v, 0, 255);
  }

  /** Returns current green color value. Value in range [0..255] */
  private int getInputGreen() {
    int v = Misc.toNumber(tfGreen.getText(), 0);
    return Misc.clamp(v, 0, 255);
  }

  /** Returns current blue color value. Value in range [0..255] */
  private int getInputBlue() {
    int v = Misc.toNumber(tfBlue.getText(), 0);
    return Misc.clamp(v, 0, 255);
  }

  /** Returns current alpha value if available. Value in range [0..255] */
  private int getInputAlpha() {
    int v = 0;
    if (isAlphaEnabled()) {
      v = Misc.toNumber(tfAlpha.getText(), 0);
    }
    return Misc.clamp(v, 0, 255);
  }

  /** Sets a new red color value. Value in range [0..255] */
  private void setInputRed(int value) {
    value = Misc.clamp(value, 0, 255);
    tfRed.setText(Integer.toString(value));
    tmpRed = value;
    updateHsbFromRgb();
    updateInputHsb();
    updatePreview();
  }

  /** Sets a new green color value. Value in range [0..255] */
  private void setInputGreen(int value) {
    value = Misc.clamp(value, 0, 255);
    tfGreen.setText(Integer.toString(value));
    tmpGreen = value;
    updateHsbFromRgb();
    updateInputHsb();
    updatePreview();
  }

  /** Sets a new blue color value. Value in range [0..255] */
  private void setInputBlue(int value) {
    value = Misc.clamp(value, 0, 255);
    tfBlue.setText(Integer.toString(value));
    tmpBlue = value;
    updateHsbFromRgb();
    updateInputHsb();
    updatePreview();
  }

  /** Sets a new alpha value if enabled. Value in range [0..255] */
  private void setInputAlpha(int value) {
    if (isAlphaEnabled()) {
      value = Misc.clamp(value, 0, 255);
      tfAlpha.setText(Integer.toString(value));
      tmpAlpha = value;
      updatePreview();
    }
  }

  /** Returns current hue value. Value in range [0..360] */
  private int getInputHue() {
    int v = Misc.toNumber(tfHue.getText(), 0);
    return Misc.clamp(v, 0, 360);
  }

  /** Returns current saturation value. Value in range [0..100] */
  private int getInputSaturation() {
    int v = Misc.toNumber(tfSat.getText(), 0);
    return Misc.clamp(v, 0, 100);
  }

  /** Returns current brightness value. Value in range [0..100] */
  private int getInputBrightness() {
    int v = Misc.toNumber(tfBri.getText(), 0);
    return Misc.clamp(v, 0, 100);
  }

  /** Sets a new hue color value. Value in range [0..360] */
  private void setInputHue(int value) {
    value = Misc.clamp(value, 0, 360);
    tfHue.setText(Integer.toString(value));
    tmpHue = value;
    updateRgbFromHsb();
    updateInputRgb();
    updatePreview();
  }

  /** Sets a new saturation color value. Value in range [0..100] */
  private void setInputSaturation(int value) {
    value = Misc.clamp(value, 0, 100);
    tfSat.setText(Integer.toString(value));
    tmpSat = value;
    updateRgbFromHsb();
    updateInputRgb();
    updatePreview();
  }

  /** Sets a new brightness color value. Value in range [0..100] */
  private void setInputBrightness(int value) {
    value = Misc.clamp(value, 0, 100);
    tfBri.setText(Integer.toString(value));
    tmpBri = value;
    updateRgbFromHsb();
    updateInputRgb();
    updatePreview();
  }

  /** Returns color value based on the main preview coordinates and the saturation value from the input field. */
  private void updateMainPreviewValue(int x, int y) {
    x = Misc.clamp(x, 0, rcMainPreview.getWidth() - 1);
    y = Misc.clamp(y, 0, rcMainPreview.getHeight() - 1);
    tmpHue = x * 360 / rcMainPreview.getWidth();
    tmpBri = 100 - (y * 100 / (rcMainPreview.getHeight() - 1));
  }

  /**
   * Returns color value based on the secondary preview coordinate and the hue/brightness values from the input fields.
   */
  private void updateSecondPreviewValue(int y) {
    y = Misc.clamp(y, 0, rcSecondPreview.getHeight() - 1);
    tmpSat = 100 - (y * 100 / (rcSecondPreview.getHeight() - 1));
  }

  /** Updates RGB values from the current HSB values. */
  private void updateRgbFromHsb() {
    final Color c = new Color(Color.HSBtoRGB(tmpHue / 360.0f, tmpSat / 100.0f, tmpBri / 100.0f));
    tmpRed = c.getRed();
    tmpGreen = c.getGreen();
    tmpBlue = c.getBlue();
  }

  /** Updates HSB values from the current RGB values. */
  private void updateHsbFromRgb() {
    float[] hsb = { 0.0f, 0.0f, 0.0f };
    Color.RGBtoHSB(tmpRed, tmpGreen, tmpBlue, hsb);
    tmpHue = Math.round(hsb[0] * 360.0f);
    tmpSat = Math.round(hsb[1] * 100.0f);
    tmpBri = Math.round(hsb[2] * 100.0f);
  }

  /** Sets current color to the specified RGB value. */
  private void updateColorValues(int value) {
    tmpRed = format.getRed(value);
    tmpGreen = format.getGreen(value);
    tmpBlue = format.getBlue(value);
    tmpAlpha = format.getAlpha(value);
    updateHsbFromRgb();
  }

  /** Update RGB input controls only. */
  private void updateInputRgb() {
    tfRed.setText(Integer.toString(tmpRed));
    if (tfRed.hasFocus()) {
      tfRed.selectAll();
    }
    tfGreen.setText(Integer.toString(tmpGreen));
    if (tfGreen.hasFocus()) {
      tfGreen.selectAll();
    }
    tfBlue.setText(Integer.toString(tmpBlue));
    if (tfBlue.hasFocus()) {
      tfBlue.selectAll();
    }
    if (isAlphaEnabled()) {
      tfAlpha.setText(Integer.toString(tmpAlpha));
      if (tfAlpha.hasFocus()) {
        tfAlpha.selectAll();
      }
    }
  }

  /** Update HSB input controls only. */
  private void updateInputHsb() {
    tfHue.setText(Integer.toString(tmpHue));
    if (tfHue.hasFocus()) {
      tfHue.selectAll();
    }
    tfSat.setText(Integer.toString(tmpSat));
    if (tfSat.hasFocus()) {
      tfSat.selectAll();
    }
    tfBri.setText(Integer.toString(tmpBri));
    if (tfBri.hasFocus()) {
      tfBri.selectAll();
    }
  }

  /** Update preview controls only. */
  private void updatePreview() {
    // update main preview
    updateMainPreview();

    // update secondary preview
    updateSecondPreview();

    // update color preview
    updateColorPreview();
  }

  /** Update controls with given color value. */
  private void updateColor() {
    updateInputRgb();
    updateInputHsb();
    updatePreview();
  }

  /** Update main (hue/brightness) preview. */
  private void updateMainPreview() {
    // drawing background
    initMainPreviewMap();

    // drawing marker
    BufferedImage image = (BufferedImage) rcMainPreview.getImage();
    if (image != null) {
      Graphics2D g = image.createGraphics();
      if (g != null) {
        try {
          int x = tmpHue * image.getWidth() / 360;
          int y = (100 - tmpBri) * image.getHeight() / 100;
          g.setColor(Color.WHITE);
          g.drawLine(x + 2, y, x + 6, y);
          g.drawLine(x - 2, y, x - 6, y);
          g.drawLine(x, y + 2, x, y + 6);
          g.drawLine(x, y - 2, x, y - 6);
        } finally {
          g.dispose();
          g = null;
        }
      }
    }
    rcMainPreview.repaint();
  }

  /** Update secondary (saturation) preview. */
  private void updateSecondPreview() {
    BufferedImage image = (BufferedImage) rcSecondPreview.getImage();
    if (image != null) {
      int width = image.getWidth();
      int height = image.getHeight();
      int type = image.getRaster().getDataBuffer().getDataType();
      if (type == DataBuffer.TYPE_INT) {
        final int[] buffer = ((DataBufferInt) image.getRaster().getDataBuffer()).getData();

        // drawing gradient and marker
        final float h = tmpHue / 360.0f;
        final float b = tmpBri / 100.0f;
        int marker = (100 - tmpSat) * height / 100;
        for (int y = 0; y < height; y++) {
          float sat = 1.0f - (y / (float)height);
          int rgb = (y == marker) ? 0xffffff : Color.HSBtoRGB(h, sat, b);
          int ofs = y * width;
          for (int x = 0; x < width; x++, ofs++) {
            buffer[ofs] = rgb;
          }
        }
      }
    }
    rcSecondPreview.repaint();
  }

  /** Update color preview box. */
  private void updateColorPreview() {
    final BufferedImage image = (BufferedImage) rcColorPreview.getImage();
    if (image != null) {
      final Graphics2D g = (Graphics2D)image.getGraphics();
      try {
        final int width = image.getWidth();
        final int height = image.getHeight();

        // drawing checkerboard as reference background
        if (isAlphaEnabled()) {
          g.setComposite(AlphaComposite.Src);
          final Color white = Color.WHITE;
          final Color gray = Color.LIGHT_GRAY;
          for (int y = 0; y < height; y += 8) {
            final int h = Math.min(8, height - y);
            for (int x = 0; x < width; x += 8) {
              final int w = Math.min(8, width - x);
              boolean odd = ((x + y) & 8) != 0;
              final Color color = odd ? gray : white;
              g.setColor(color);
              g.fillRect(x, y, w, h);
            }
          }
        }

        // drawing color
        g.setComposite(AlphaComposite.SrcOver);
        final Color color = new Color(ColorFormat.BGRA.getRGBA(tmpRed, tmpGreen, tmpBlue, tmpAlpha), isAlphaEnabled());
        g.setColor(color);
        g.fillRect(0, 0, width, height);
      } finally {
        g.dispose();
      }
    }
    rcColorPreview.repaint();
  }

  /** Update main preview background map. */
  private void initMainPreviewMap() {
    BufferedImage image = (BufferedImage) rcMainPreview.getImage();
    if (image != null) {
      int width = image.getWidth();
      int height = image.getHeight();
      int[] buffer = ((DataBufferInt) image.getRaster().getDataBuffer()).getData();
      if (buffer != null) {
        final float s = tmpSat / 100.0f;
        for (int y = 0; y < height; y++) {
          final float b = 1.0f - (y / (float) height);
          for (int x = 0; x < width; x++) {
            final float h = x / (float) width;
            final int rgb = getHsbValue(h, s, b);
            buffer[y * width + x] = (format.getRed(rgb) << 16) | (format.getGreen(rgb) << 8) | format.getBlue(rgb);
          }
        }
      }
    }
  }

  // -------------------------- INNER CLASSES --------------------------

  private class Listeners implements MouseListener, MouseMotionListener, FocusListener, KeyListener {
    private boolean dragMainEnabled;
    private boolean dragSecondEnabled;

    public Listeners() {
      dragMainEnabled = false;
      dragSecondEnabled = false;
    }

    /** Indicates whether the mouse button is pressed while moving the cursor inside the main preview canvas. */
    private boolean isDragMainEnabled() {
      return dragMainEnabled;
    }

    /** Sets the state of the mouse buttons tate for the main preview canvas. */
    private void setDragMainEnabled(boolean set) {
      dragMainEnabled = set;
      if (dragMainEnabled) {
        dragSecondEnabled = false;
      }
    }

    /** Indicates whether the mouse button is pressed while moving the cursor inside the secondary preview canvas. */
    private boolean isDragSecondEnabled() {
      return dragSecondEnabled;
    }

    /** Sets the state of the mouse buttons tate for the secondary preview canvas. */
    private void setDragSecondEnabled(boolean set) {
      dragSecondEnabled = set;
      if (dragSecondEnabled) {
        dragMainEnabled = false;
      }
    }

    // --------------------- Begin Interface MouseListener ---------------------

    @Override
    public void mouseClicked(MouseEvent e) {
    }

    @Override
    public void mousePressed(MouseEvent e) {
      if (e.getSource() == rcMainPreview && e.getButton() == MouseEvent.BUTTON1) {
        updateMainPreviewValue(e.getX(), e.getY());
        updateRgbFromHsb();
        updateColor();
        setDragMainEnabled(true);
      } else if (e.getSource() == rcSecondPreview && e.getButton() == MouseEvent.BUTTON1) {
        updateSecondPreviewValue(e.getY());
        updateRgbFromHsb();
        updateColor();
        setDragSecondEnabled(true);
      }
    }

    @Override
    public void mouseReleased(MouseEvent e) {
      if (e.getSource() == rcMainPreview && e.getButton() == MouseEvent.BUTTON1) {
        setDragMainEnabled(false);
      } else if (e.getSource() == rcSecondPreview && e.getButton() == MouseEvent.BUTTON1) {
        setDragSecondEnabled(false);
      }
    }

    @Override
    public void mouseEntered(MouseEvent e) {
    }

    @Override
    public void mouseExited(MouseEvent e) {
      if (e.getSource() == rcMainPreview && e.getButton() == MouseEvent.BUTTON1) {
        setDragMainEnabled(false);
        updateMainPreviewValue(e.getX(), e.getY());
        updateRgbFromHsb();
        updateColor();
      } else if (e.getSource() == rcSecondPreview && e.getButton() == MouseEvent.BUTTON1) {
        setDragSecondEnabled(false);
        updateSecondPreviewValue(e.getY());
        updateRgbFromHsb();
        updateColor();
      }
    }

    // --------------------- End Interface MouseListener ---------------------

    // --------------------- Begin Interface MouseMotionListener ---------------------

    @Override
    public void mouseDragged(MouseEvent e) {
      if (e.getSource() == rcMainPreview && isDragMainEnabled()) {
        updateMainPreviewValue(e.getX(), e.getY());
        updateRgbFromHsb();
        updateColor();
      } else if (e.getSource() == rcSecondPreview && isDragSecondEnabled()) {
        updateSecondPreviewValue(e.getY());
        updateRgbFromHsb();
        updateColor();
      }
    }

    @Override
    public void mouseMoved(MouseEvent e) {
    }

    // --------------------- End Interface MouseMotionListener ---------------------

    // --------------------- Begin Interface FocusListener ---------------------

    @Override
    public void focusGained(FocusEvent e) {
      if (e.getSource() instanceof JTextField) {
        ((JTextField) e.getSource()).selectAll();
      }
    }

    @Override
    public void focusLost(FocusEvent e) {
      if (e.getSource() == tfRed) {
        int v = validateNumberInput(tfRed, tmpRed, 0, 255);
        if (v != tmpRed) {
          setInputRed(v);
        }
      } else if (e.getSource() == tfGreen) {
        int v = validateNumberInput(tfGreen, tmpGreen, 0, 255);
        if (v != tmpGreen) {
          setInputGreen(v);
        }
      } else if (e.getSource() == tfBlue) {
        int v = validateNumberInput(tfBlue, tmpBlue, 0, 255);
        if (v != tmpBlue) {
          setInputBlue(v);
        }
      } else if (tfAlpha != null && e.getSource() == tfAlpha) {
        int v = validateNumberInput(tfAlpha, tmpAlpha, 0, 255);
        if (v != tmpAlpha) {
          setInputAlpha(v);
        }
      } else if (e.getSource() == tfHue) {
        int v = validateNumberInput(tfHue, tmpHue, 0, 360);
        if (v != tmpHue) {
          setInputHue(v);
        }
      } else if (e.getSource() == tfSat) {
        int v = validateNumberInput(tfSat, tmpSat, 0, 100);
        if (v != tmpSat) {
          setInputSaturation(v);
        }
      } else if (e.getSource() == tfBri) {
        int v = validateNumberInput(tfBri, tmpBri, 0, 100);
        if (v != tmpBri) {
          setInputBrightness(v);
        }
      }
    }

    // --------------------- End Interface FocusListener ---------------------

    // --------------------- Begin Interface KeyListener ---------------------

    @Override
    public void keyTyped(KeyEvent e) {
    }

    @Override
    public void keyPressed(KeyEvent e) {
      final Integer incVal;
      final int factor = (e.getModifiersEx() & KeyEvent.SHIFT_DOWN_MASK) != 0 ? 10 : 1;
      switch (e.getKeyCode()) {
        case KeyEvent.VK_UP:
        case KeyEvent.VK_KP_UP:
          incVal = 1 * factor;
          break;
        case KeyEvent.VK_DOWN:
        case KeyEvent.VK_KP_DOWN:
          incVal = -1 * factor;
          break;
        default:
          incVal = null;
      }

      if (e.getSource() == tfRed) {
        if (incVal != null) {
          setInputRed(getInputRed() + incVal);
          tfRed.selectAll();
        }
      } else if (e.getSource() == tfGreen) {
        if (incVal != null) {
          setInputGreen(getInputGreen() + incVal);
          tfGreen.selectAll();
        }
      } else if (e.getSource() == tfBlue) {
        if (incVal != null) {
          setInputBlue(getInputBlue() + incVal);
          tfBlue.selectAll();
        }
      } else if (tfAlpha != null && e.getSource() == tfAlpha) {
        if (incVal != null) {
          setInputAlpha(getInputAlpha() + incVal);
          tfAlpha.selectAll();
        }
      } else if (e.getSource() == tfHue) {
        if (incVal != null) {
          setInputHue(getInputHue() + incVal);
          tfHue.selectAll();
        }
      } else if (e.getSource() == tfSat) {
        if (incVal != null) {
          setInputSaturation(getInputSaturation() + incVal);
          tfSat.selectAll();
        }
      } else if (e.getSource() == tfBri) {
        if (incVal != null) {
          setInputBrightness(getInputBrightness() + incVal);
          tfBri.selectAll();
        }
      }
    }

    @Override
    public void keyReleased(KeyEvent e) {
    }

    // --------------------- End Interface KeyListener ---------------------
  }
}
