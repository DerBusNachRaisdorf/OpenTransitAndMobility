export function kilometersToReadableText(km) {
  if (km < 1) {
    return `${(km * 1000).toFixed(0)} m`;
  } else {
    return `${km.toFixed(3)} km`;
  }
}

export class SafariCompatibleDate extends Date {
  constructor(value) {
    super(value.replace(/\s/, "T"));
  }
}

/**
 * calculates the difference between two Date-objects in minutes.
 * @param {Date} start first Date-object.
 * @param {Date} end second Date-object.
 * @returns The difference between 'end' and 'start' in Minutes
 */
export function timeDifferenceMinutes(start, end) {
  return Math.floor((end.getTime() - start.getTime()) / 60000);
}

/**
 * subtracts a specified amount of hours from a Date-object.
 * @param {Date} date The date to subtract from.
 * @param {number} hours The number of hours to subtract.
 * @returns A copy of the Date-object with the according amount of hours subtracted.
 */
export function subtractHours(date, hours) {
  const dateCopy = new Date(date);
  dateCopy.setHours(dateCopy.getHours() - hours);
  return dateCopy;
}

/**
 * subtracts a specified amount of days from a Date-object.
 * @param {Date} date The date to subtract from.
 * @param {number} days The number of days to subtract.
 * @returns A copy of the Date-object with the according amount of days subtracted.
 */
export function subtractDays(date, days) {
  let result = new Date(date);
  result.setDate(result.getDate() - days);
  return result;
}

/**
 * Formats a dateTime object as HH:MM.
 * @param {SafariCompatibleDate} num The number to format.
 * @returns A two-digit string of the number.
 */
export function timeString(dateTime) {
  /**
   * formats a number always as a two-digit string.
   * @param {Number} num The number to format.
   * @returns A two-digit string of the number.
   */
  function leadingZeros(num) {
    return ("00" + num).slice(-2);
  }

  const hoursString = leadingZeros(dateTime.getHours());
  const minutesString = leadingZeros(dateTime.getMinutes());

  return `${hoursString}:${minutesString}`;
}
